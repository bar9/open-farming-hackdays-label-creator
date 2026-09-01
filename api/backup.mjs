#!/usr/bin/env node
// Sichern, Wiederherstellen und Prüfen aller Kurz-Links.
//
// Kurz-Links sind dauerhafter Zustand: Einer auf einem gedruckten Etikett
// lässt sich nicht mehr ändern. Geht die Datenbank verloren, sind alle
// ausgedruckten Etiketten unbrauchbar — die lange URL trägt ihre Daten selbst
// und ist davon nicht betroffen, der Kurz-Link nicht. Der Bestand ist winzig
// (~1 KB je Eintrag), ein vollständiger Export kostet also fast nichts.
//
// Der Import erlaubt zusätzlich den Anbieterwechsel: exportieren, die
// Umgebungsvariablen umstellen, importieren. Die Codes bleiben gleich, weil
// sie aus der URL abgeleitet sind.
//
//   node api/backup.mjs export > links.json
//   node api/backup.mjs import links.json
//   node api/backup.mjs verify [links.json]
//
// Zugangsdaten wie beim Selbsttest über Umgebungsvariablen, z.B.
//   TURSO_DATABASE_URL=… TURSO_AUTH_TOKEN=… node api/backup.mjs export

import { readFile } from "node:fs/promises";

import { listAll, storageBackend, storeIfAbsent } from "./_storage.mjs";
import { SHORT_BASE, isAllowedTarget } from "./_lib.mjs";

const USAGE =
  "Sichern und Wiederherstellen der Kurz-Links.\n\n" +
  "  node api/backup.mjs export > links.json\n" +
  "  node api/backup.mjs import links.json\n" +
  "  node api/backup.mjs verify [links.json]\n\n" +
  "export/import brauchen TURSO_DATABASE_URL/TURSO_AUTH_TOKEN oder\n" +
  "KV_REST_API_URL/KV_REST_API_TOKEN. `verify <datei.json>` prüft nur die\n" +
  "öffentliche Seite und läuft ohne Zugangsdaten.";

const [command, file] = process.argv.slice(2);

// Ohne (oder mit unbekanntem) Befehl zuerst die Hilfe zeigen, statt über
// fehlende Zugangsdaten zu stolpern.
if (!["export", "import", "verify"].includes(command)) {
  console.error(USAGE);
  process.exit(2);
}

// `verify <datei>` prüft nur die öffentliche Seite und braucht deshalb keine
// Zugangsdaten — gerade das macht es als Überwachung brauchbar, etwa aus CI
// oder von einem Rechner ohne Datenbankzugang.
const backend = storageBackend();
if (!backend && !(command === "verify" && file)) {
  console.error(
    "Kein Speicher konfiguriert. Setze TURSO_DATABASE_URL/TURSO_AUTH_TOKEN\n" +
      "oder KV_REST_API_URL/KV_REST_API_TOKEN.\n" +
      "(`verify <datei.json>` läuft auch ohne Zugangsdaten.)"
  );
  process.exit(2);
}

async function readLinks(path) {
  const data = JSON.parse(await readFile(path, "utf8"));
  return data.links ?? [];
}

/** Export nach stdout, damit er sich umleiten und versionieren lässt;
 *  Diagnose nach stderr, um die JSON-Ausgabe sauber zu halten. */
async function runExport() {
  const links = await listAll();
  console.error(`${links.length} Einträge aus ${backend} gelesen.`);
  console.log(
    JSON.stringify(
      { exported_at: new Date().toISOString(), backend, count: links.length, links },
      null,
      2
    )
  );
  return 0;
}

async function runImport() {
  if (!file) {
    console.error("Aufruf: node api/backup.mjs import <datei.json>");
    return 2;
  }
  let written = 0;
  let existing = 0;
  let rejected = 0;
  for (const link of await readLinks(file)) {
    // Auch beim Import gilt die Allowlist: eine manipulierte Sicherungsdatei
    // darf keine fremden Ziele einschleusen.
    if (!isAllowedTarget(link.url)) {
      console.error(`  übersprungen (fremdes Ziel): ${link.code}`);
      rejected++;
    } else if (await storeIfAbsent(link.code, link.url)) {
      written++;
    } else {
      existing++;
    }
  }
  console.error(
    `Import nach ${backend}: ${written} neu, ${existing} bereits vorhanden, ${rejected} abgelehnt.`
  );
  return 0;
}

/** Prüft gegen die öffentliche Seite statt gegen die Datenbank: nur so ist
 *  belegt, was ein Empfänger eines gedruckten Links tatsächlich erlebt. */
async function runVerify() {
  const links = file ? await readLinks(file) : await listAll();
  let ok = 0;
  let bad = 0;
  for (const link of links) {
    const response = await fetch(`${SHORT_BASE}/s/${link.code}`, { redirect: "manual" });
    const location = response.headers.get("location");
    if (response.status === 301 && location === link.url) {
      ok++;
    } else {
      bad++;
      console.error(`  FEHLT: /s/${link.code} -> HTTP ${response.status}`);
    }
  }
  console.error(`${ok} Links funktionieren, ${bad} fehlerhaft.`);
  return bad === 0 ? 0 : 1;
}

const commands = { export: runExport, import: runImport, verify: runVerify };
process.exit(await commands[command]());
