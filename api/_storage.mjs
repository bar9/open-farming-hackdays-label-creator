// Speicher-Anbindung für den Kurz-Link-Dienst.
//
// Zwei Anbieter, beide über HTTPS statt eines Verbindungsprotokolls:
// Vercel-Funktionen leben nur Millisekunden und können keine dauerhafte
// TCP-Verbindung halten, wie klassisches Redis oder Postgres sie erwarten.
//
//   Turso (libSQL)  SQLite über HTTPS, erkannt an TURSO_DATABASE_URL
//   Upstash Redis   Key/Value über HTTPS, erkannt an KV_REST_API_URL
//
// Die Wahl trifft allein die Umgebung; im Code steht keine Festlegung. Ein
// Anbieterwechsel ist damit eine Frage der Projekteinstellungen, nicht des
// Quelltexts — wichtig, weil geteilte Kurz-Links auf gedruckten Etiketten
// landen und den Anbieter überleben müssen.
//
// Zugangsdaten kommen ausschliesslich aus Umgebungsvariablen und bleiben
// damit serverseitig. Im WASM-Frontend wäre jedes Geheimnis auslesbar.

/** Name des aktiven Anbieters, oder null wenn keiner konfiguriert ist. */
export function storageBackend() {
  if (process.env.TURSO_DATABASE_URL) return "turso";
  if (process.env.KV_REST_API_URL || process.env.UPSTASH_REDIS_REST_URL) {
    return "upstash";
  }
  return null;
}

function noStorageConfigured() {
  return new Error(
    "Kein Speicher konfiguriert: TURSO_DATABASE_URL oder KV_REST_API_URL setzen"
  );
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
 *  `close` wird immer mitgeschickt: offene Verbindungen laufen sonst zehn
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

/** Ob ein Fehler nur bedeutet, dass noch nie etwas gespeichert wurde.
 *  Vor dem ersten Kürzen existiert die Tabelle nicht; das ist ein leerer
 *  Bestand, kein Fehler. */
function isMissingTable(error) {
  return /no such table/i.test(String(error?.message ?? ""));
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
    if (isMissingTable(error)) return null;
    throw error;
  }
  const rows = results[0]?.response?.result?.rows ?? [];
  return rows.length > 0 ? rows[0][0].value : null;
}

async function tursoListAll() {
  let results;
  try {
    results = await tursoExecute([
      { sql: "SELECT code, url, created_at FROM links ORDER BY created_at" },
    ]);
  } catch (error) {
    if (isMissingTable(error)) return [];
    throw error;
  }
  const rows = results[0]?.response?.result?.rows ?? [];
  return rows.map((row) => ({
    code: row[0].value,
    url: row[1].value,
    created_at: Number(row[2].value),
  }));
}

// -------------------------------------------------------------- Upstash ---

/** Schlüssel-Präfix, damit die Kurz-Links in einer mitbenutzten Redis-Instanz
 *  nicht mit fremden Schlüsseln kollidieren. */
const UPSTASH_PREFIX = "s:";

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

async function upstashListAll() {
  // SCAN in Schritten, damit auch grosse Bestände vollständig durchlaufen.
  const entries = [];
  let cursor = "0";
  do {
    const [next, keys] = await upstashCommand([
      "SCAN",
      cursor,
      "MATCH",
      `${UPSTASH_PREFIX}*`,
      "COUNT",
      "200",
    ]);
    cursor = next;
    for (const key of keys) {
      const url = await upstashCommand(["GET", key]);
      // Redis kennt kein Anlagedatum; der Export lässt das Feld dann leer.
      if (url !== null) {
        entries.push({ code: key.slice(UPSTASH_PREFIX.length), url, created_at: null });
      }
    }
  } while (cursor !== "0");
  return entries;
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
      return (
        (await upstashCommand(["SET", `${UPSTASH_PREFIX}${code}`, url, "NX"])) === "OK"
      );
    default:
      throw noStorageConfigured();
  }
}

/** Ziel-URL zu einem Code, oder null. */
export async function lookup(code) {
  switch (storageBackend()) {
    case "turso":
      return await tursoLookup(code);
    case "upstash":
      return await upstashCommand(["GET", `${UPSTASH_PREFIX}${code}`]);
    default:
      throw noStorageConfigured();
  }
}

/** Alle Einträge als `{ code, url, created_at }`, für Sicherung und
 *  Anbieterwechsel (siehe backup.mjs).
 *
 *  Anbieterabhängig, weil Auflisten die einzige Operation ist, die sich nicht
 *  auf einen gemeinsamen Aufruf abbilden lässt: SQL kennt SELECT, Redis
 *  braucht SCAN. */
export async function listAll() {
  switch (storageBackend()) {
    case "turso":
      return await tursoListAll();
    case "upstash":
      return await upstashListAll();
    default:
      throw noStorageConfigured();
  }
}
