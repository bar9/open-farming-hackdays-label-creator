#!/usr/bin/env node
// VORÜBERGEHEND: liest alle Kurz-Links aus der alten Turso-Datenbank.
//
// Nur für die Umstellung auf den eigenen Server gedacht (siehe
// ops/migration/sync-links.sh). Nach der DNS-Umstellung und dem letzten
// Abgleich zusammen mit diesem Verzeichnis löschen.
//
// Rein lesend: eine einzige SELECT-Anweisung über die HTTP-Pipeline-API.
// Ausgabe im Format, das `shortener import` liest.
//
//   TURSO_DATABASE_URL=… TURSO_AUTH_TOKEN=… node ops/migration/turso-export.mjs > links.json

const raw = process.env.TURSO_DATABASE_URL ?? "";
const token = process.env.TURSO_AUTH_TOKEN ?? "";
if (!raw || !token) {
  console.error("TURSO_DATABASE_URL und TURSO_AUTH_TOKEN setzen.");
  process.exit(2);
}
// Die HTTP-API erwartet https:// statt der SDK-Schemata libsql:// bzw. turso://.
const url = raw.replace(/^(libsql|turso):\/\//, "https://").replace(/\/$/, "");

const response = await fetch(`${url}/v2/pipeline`, {
  method: "POST",
  headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
  body: JSON.stringify({
    requests: [
      {
        type: "execute",
        stmt: { sql: "SELECT code, url, created_at FROM links ORDER BY created_at, code" },
      },
      { type: "close" },
    ],
  }),
});
if (!response.ok) {
  console.error(`Turso HTTP ${response.status}: ${await response.text()}`);
  process.exit(1);
}
const body = await response.json();
// Die Pipeline liefert HTTP 200 auch dann, wenn die Anweisung scheitert.
const result = body.results?.[0];
if (result?.type !== "ok") {
  console.error(`Turso: ${result?.error?.message ?? "unbekannter Fehler"}`);
  process.exit(1);
}
const links = result.response.result.rows.map((row) => ({
  code: row[0].value,
  url: row[1].value,
  created_at: Number(row[2].value),
}));
console.error(`${links.length} Einträge aus Turso gelesen.`);
console.log(
  JSON.stringify(
    { exported_at: new Date().toISOString(), backend: "turso", count: links.length, links },
    null,
    2
  )
);
