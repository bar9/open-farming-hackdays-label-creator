// Speicher-Anbindung für den Kurz-Link-Dienst.
//
// Unterstützt zwei Anbieter aus dem Vercel Marketplace, weil beide über
// HTTPS ansprechbar sind: Vercel-Funktionen leben nur Millisekunden und
// können keine dauerhafte TCP-Verbindung halten, wie sie klassisches Redis
// oder Postgres erwarten.
//
//   * Turso (libSQL)  — SQLite über HTTPS, erkannt an TURSO_DATABASE_URL
//   * Upstash Redis   — Key/Value über HTTPS, erkannt an KV_REST_API_URL
//
// Welcher benutzt wird, entscheiden allein die gesetzten Umgebungsvariablen;
// im Code steht keine Festlegung. Ein Anbieterwechsel ist damit eine Frage
// der Projekteinstellungen, nicht des Quelltexts — wichtig, weil geteilte
// Kurz-Links auf gedruckten Etiketten landen und den Anbieter überleben
// müssen.
//
// Die Zugangsdaten kommen ausschliesslich aus Umgebungsvariablen und liegen
// damit serverseitig. Im WASM-Frontend wäre jedes Geheimnis auslesbar
// (`strings …wasm | grep`).

/** Name des aktiven Anbieters, für Fehlermeldungen und Diagnose. */
export function storageBackend() {
  if (process.env.TURSO_DATABASE_URL) return "turso";
  if (process.env.KV_REST_API_URL || process.env.UPSTASH_REDIS_REST_URL) {
    return "upstash";
  }
  return null;
}

// ---------------------------------------------------------------- Turso ---

function tursoConfig() {
  // Die Vercel-Integration setzt TURSO_DATABASE_URL; die HTTP-API erwartet
  // aber https:// statt der SDK-Schemata libsql:// bzw. turso://.
  const raw = process.env.TURSO_DATABASE_URL ?? "";
  const url = raw.replace(/^(libsql|turso):\/\//, "https://").replace(/\/$/, "");
  const token = process.env.TURSO_AUTH_TOKEN;
  if (!url || !token) {
    throw new Error("Turso unvollständig: TURSO_DATABASE_URL/TURSO_AUTH_TOKEN");
  }
  return { url, token };
}

/** Eine Folge von SQL-Anweisungen über die Pipeline-API ausführen.
 *
 *  `close` wird immer mitgeschickt: offene Verbindungen laufen sonst 10
 *  Sekunden im Leerlauf weiter, und bei einer Funktion pro Anfrage würden
 *  sich die schnell summieren. */
async function tursoExecute(statements) {
  const { url, token } = tursoConfig();
  const response = await fetch(`${url}/v2/pipeline`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      requests: [
        ...statements.map((stmt) => ({ type: "execute", stmt })),
        { type: "close" },
      ],
    }),
  });
  if (!response.ok) {
    throw new Error(`Turso HTTP ${response.status}: ${await response.text()}`);
  }
  const body = await response.json();
  // Die Pipeline liefert HTTP 200 auch dann, wenn eine einzelne Anweisung
  // scheitert — der Fehler steckt im jeweiligen Ergebnis.
  const failed = (body.results ?? []).find((r) => r?.type === "error");
  if (failed) {
    throw new Error(`Turso: ${failed.error?.message ?? "unbekannter Fehler"}`);
  }
  return body.results ?? [];
}

/** Tabelle anlegen, falls sie fehlt.
 *
 *  Bewusst bei jedem Schreibzugriff statt als separater Migrationsschritt:
 *  der Dienst hat genau eine Tabelle, und so ist eine frisch angelegte
 *  Datenbank ohne manuelles Zutun sofort benutzbar. Für Lesezugriffe wird
 *  sie nicht gebraucht (fehlende Tabelle = unbekannter Code = 404). */
const TURSO_SCHEMA = `CREATE TABLE IF NOT EXISTS links (
  code TEXT PRIMARY KEY,
  url TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
)`;

async function tursoStoreIfAbsent(code, url) {
  // INSERT ... ON CONFLICT DO NOTHING: nur schreiben, wenn der Code frei ist.
  // affected_row_count unterscheidet dann "neu angelegt" von "schon belegt",
  // ohne dass zwischen Prüfen und Schreiben eine Lücke entsteht.
  const results = await tursoExecute([
    { sql: TURSO_SCHEMA },
    {
      sql: "INSERT INTO links (code, url) VALUES (?, ?) ON CONFLICT(code) DO NOTHING",
      args: [
        { type: "text", value: code },
        { type: "text", value: url },
      ],
    },
  ]);
  const insert = results[1]?.response?.result;
  return (insert?.affected_row_count ?? 0) > 0;
}

async function tursoLookup(code) {
  let results;
  try {
    results = await tursoExecute([
      {
        sql: "SELECT url FROM links WHERE code = ?",
        args: [{ type: "text", value: code }],
      },
    ]);
  } catch (error) {
    // Vor dem ersten Kürzen existiert die Tabelle noch nicht; das ist kein
    // Fehler, sondern schlicht ein unbekannter Code.
    if (/no such table/i.test(String(error.message))) return null;
    throw error;
  }
  const rows = results[0]?.response?.result?.rows ?? [];
  return rows.length > 0 ? rows[0][0].value : null;
}

// -------------------------------------------------------------- Upstash ---

function upstashConfig() {
  // Die Vercel-Upstash-Integration setzt die KV_REST_API_*-Namen; die
  // UPSTASH_*-Namen sind der Fallback beim manuellen Eintragen.
  const url = process.env.KV_REST_API_URL ?? process.env.UPSTASH_REDIS_REST_URL;
  const token =
    process.env.KV_REST_API_TOKEN ?? process.env.UPSTASH_REDIS_REST_TOKEN;
  if (!url || !token) {
    throw new Error("Upstash unvollständig: KV_REST_API_URL/TOKEN");
  }
  return { url: url.replace(/\/$/, ""), token };
}

async function upstashCommand(command) {
  const { url, token } = upstashConfig();
  const response = await fetch(url, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(command),
  });
  if (!response.ok) {
    throw new Error(`Redis HTTP ${response.status}: ${await response.text()}`);
  }
  const body = await response.json();
  if (body.error) throw new Error(`Redis: ${body.error}`);
  return body.result;
}

// ------------------------------------------------------------ Schnittstelle

/** Eintrag anlegen, wenn der Code noch frei ist.
 *  true = geschrieben, false = Code bereits belegt.
 *  Bewusst ohne Ablaufdatum: geteilte und gedruckte Links dürfen nicht
 *  verschwinden. */
export async function storeIfAbsent(code, url) {
  switch (storageBackend()) {
    case "turso":
      return await tursoStoreIfAbsent(code, url);
    case "upstash":
      // NX schreibt nur, wenn der Schlüssel noch nicht existiert.
      return (await upstashCommand(["SET", `s:${code}`, url, "NX"])) === "OK";
    default:
      throw new Error(
        "Kein Speicher konfiguriert: TURSO_DATABASE_URL oder KV_REST_API_URL setzen"
      );
  }
}

/** Ziel-URL zu einem Code, oder null. */
export async function lookup(code) {
  switch (storageBackend()) {
    case "turso":
      return await tursoLookup(code);
    case "upstash":
      return await upstashCommand(["GET", `s:${code}`]);
    default:
      throw new Error(
        "Kein Speicher konfiguriert: TURSO_DATABASE_URL oder KV_REST_API_URL setzen"
      );
  }
}
