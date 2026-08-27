#!/usr/bin/env node
// Sichern und Wiederherstellen aller Kurz-Links.
//
// Warum es das braucht: Seit dem eigenen Kurz-Link-Dienst gibt es dauerhaften
// Zustand. Ein Kurz-Link auf einem gedruckten Etikett lässt sich nicht mehr
// ändern — geht die Datenbank verloren (Konto gekündigt, Gratis-Kontingent
// beendet, Anbieter eingestellt), sind alle ausgedruckten Etiketten
// unbrauchbar. Die lange URL trägt ihre Daten selbst und ist davon nicht
// betroffen; der Kurz-Link nicht.
//
// Der Datenbestand ist winzig (~1 KB pro Eintrag), ein vollständiger Export
// kostet also fast nichts. Import erlaubt zusätzlich den Anbieterwechsel:
// Export bei Turso, Variablen auf Upstash umstellen, Import — die Links
// bleiben dieselben, weil der Code aus der URL abgeleitet ist.
//
// Aufruf:
//   node api/backup.mjs export > links.json
//   node api/backup.mjs import links.json
//   node api/backup.mjs verify links.json   # gegen die Live-Seite prüfen
//
// Zugangsdaten wie beim Selbsttest über Umgebungsvariablen, z.B.
//   TURSO_DATABASE_URL=… TURSO_AUTH_TOKEN=… node api/backup.mjs export

import { storageBackend, storeIfAbsent, lookup } from "./_storage.mjs";
import { SHORT_BASE, isAllowedTarget } from "./_lib.mjs";

const backend = storageBackend();
// `verify <datei>` prüft nur die öffentliche Seite und braucht deshalb keine
// Zugangsdaten — gerade das macht es als Überwachung brauchbar, etwa aus CI
// oder von einem Rechner ohne Datenbankzugang. Alle anderen Befehle brauchen
// den Speicher.
const command_ = process.argv[2];
// Ohne (oder mit unbekanntem) Befehl zuerst die Hilfe zeigen, statt über
// fehlende Zugangsdaten zu stolpern.
if (!["export", "import", "verify"].includes(command_)) {
  console.error(
    "Sichern und Wiederherstellen der Kurz-Links.\n\n" +
      "  node api/backup.mjs export > links.json\n" +
      "  node api/backup.mjs import links.json\n" +
      "  node api/backup.mjs verify [links.json]\n\n" +
      "export/import brauchen TURSO_DATABASE_URL/TURSO_AUTH_TOKEN oder\n" +
      "KV_REST_API_URL/KV_REST_API_TOKEN. `verify <datei.json>` prüft nur die\n" +
      "öffentliche Seite und läuft ohne Zugangsdaten."
  );
  process.exit(2);
}
const needsStorage = !(command_ === "verify" && process.argv[3]);
if (!backend && needsStorage) {
  console.error(
    "Kein Speicher konfiguriert. Setze TURSO_DATABASE_URL/TURSO_AUTH_TOKEN\n" +
      "oder KV_REST_API_URL/KV_REST_API_TOKEN.\n" +
      "(`verify <datei.json>` läuft auch ohne Zugangsdaten.)"
  );
  process.exit(2);
}

/** Alle Einträge auflisten.
 *
 *  Anbieterabhängig, weil das Auflisten die einzige Operation ist, die sich
 *  nicht sinnvoll hinter der gemeinsamen Schnittstelle abstrahieren lässt:
 *  SQL kennt SELECT, Redis braucht SCAN. */
async function listAll() {
  if (backend === "turso") {
    const url = (process.env.TURSO_DATABASE_URL ?? "")
      .replace(/^(libsql|turso):\/\//, "https://")
      .replace(/\/$/, "");
    const response = await fetch(`${url}/v2/pipeline`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${process.env.TURSO_AUTH_TOKEN}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        requests: [
          {
            type: "execute",
            stmt: { sql: "SELECT code, url, created_at FROM links ORDER BY created_at" },
          },
          { type: "close" },
        ],
      }),
    });
    if (!response.ok) throw new Error(`Turso HTTP ${response.status}`);
    const body = await response.json();
    const failed = (body.results ?? []).find((r) => r?.type === "error");
    // Vor dem ersten Kürzen existiert die Tabelle nicht — das ist ein leerer
    // Bestand, kein Fehler.
    if (failed) {
      if (/no such table/i.test(failed.error?.message ?? "")) return [];
      throw new Error(`Turso: ${failed.error?.message}`);
    }
    const rows = body.results[0]?.response?.result?.rows ?? [];
    return rows.map((r) => ({
      code: r[0].value,
      url: r[1].value,
      created_at: Number(r[2].value),
    }));
  }

  // Upstash: SCAN in Schritten, damit auch grosse Bestände durchlaufen.
  const url = (process.env.KV_REST_API_URL ?? process.env.UPSTASH_REDIS_REST_URL).replace(/\/$/, "");
  const token = process.env.KV_REST_API_TOKEN ?? process.env.UPSTASH_REDIS_REST_TOKEN;
  const call = async (cmd) => {
    const r = await fetch(url, {
      method: "POST",
      headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
      body: JSON.stringify(cmd),
    });
    if (!r.ok) throw new Error(`Redis HTTP ${r.status}`);
    const b = await r.json();
    if (b.error) throw new Error(`Redis: ${b.error}`);
    return b.result;
  };
  const entries = [];
  let cursor = "0";
  do {
    const [next, keys] = await call(["SCAN", cursor, "MATCH", "s:*", "COUNT", "200"]);
    cursor = next;
    for (const key of keys) {
      const value = await call(["GET", key]);
      if (value !== null) entries.push({ code: key.slice(2), url: value, created_at: null });
    }
  } while (cursor !== "0");
  return entries;
}

const [command, file] = process.argv.slice(2);

if (command === "export") {
  const links = await listAll();
  // Nach stdout, damit sich der Export umleiten und versionieren lässt;
  // Diagnose nach stderr, um die JSON-Ausgabe sauber zu halten.
  console.error(`${links.length} Einträge aus ${backend} gelesen.`);
  console.log(
    JSON.stringify(
      { exported_at: new Date().toISOString(), backend, count: links.length, links },
      null,
      2
    )
  );
} else if (command === "import") {
  if (!file) {
    console.error("Aufruf: node api/backup.mjs import <datei.json>");
    process.exit(2);
  }
  const { readFile } = await import("node:fs/promises");
  const data = JSON.parse(await readFile(file, "utf8"));
  let written = 0;
  let existing = 0;
  let rejected = 0;
  for (const link of data.links ?? []) {
    // Auch beim Import gilt die Allowlist: eine manipulierte Sicherungsdatei
    // darf keine fremden Ziele einschleusen.
    if (!isAllowedTarget(link.url)) {
      console.error(`  übersprungen (fremdes Ziel): ${link.code}`);
      rejected++;
      continue;
    }
    if (await storeIfAbsent(link.code, link.url)) written++;
    else existing++;
  }
  console.error(
    `Import nach ${backend}: ${written} neu, ${existing} bereits vorhanden, ${rejected} abgelehnt.`
  );
} else if (command === "verify") {
  // Prüft gegen die öffentliche Seite, nicht gegen die Datenbank: nur so ist
  // belegt, dass ein gedruckter Link für einen Empfänger wirklich funktioniert.
  const source = file
    ? JSON.parse(await (await import("node:fs/promises")).readFile(file, "utf8")).links
    : await listAll();
  let ok = 0;
  let bad = 0;
  for (const link of source) {
    const response = await fetch(`${SHORT_BASE}/s/${link.code}`, { redirect: "manual" });
    const location = response.headers.get("location");
    if (response.status === 301 && location === link.url) ok++;
    else {
      bad++;
      console.error(`  FEHLT: /s/${link.code} -> HTTP ${response.status}`);
    }
  }
  console.error(`${ok} Links funktionieren, ${bad} fehlerhaft.`);
  process.exit(bad === 0 ? 0 : 1);
}
