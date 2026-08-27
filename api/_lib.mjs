// Gemeinsame Helfer für die beiden Shortener-Funktionen.
//
// Warum es diesen Endpunkt überhaupt gibt: Fremd-Shortener sind für Declarino
// unbrauchbar geworden. da.gd und spoo.me werden von Swisscoms DNS auf einen
// Sperrserver umgebogen (195.186.4.x), weil die Kategorie "URL-Shortener" als
// Phishing-Vektor gilt; der Browser meldet dann nur "Failed to fetch". is.gd
// und v.gd fielen mit "Error, database insert failed" aus, und tinyurl zeigt
// sporadisch Warn-Zwischenseiten. Ein Kurz-Link unter declarino.ch steht auf
// keiner solchen Liste, weil die Domain eine eigene Reputation hat.

import { createHash } from "node:crypto";

/** Basis der Kurz-Links. Immer Produktion: ein in Staging oder lokal
 *  erzeugter Link soll auch verschickt funktionieren. */
export const SHORT_BASE = "https://www.declarino.ch";

/** Hosts, deren URLs gekürzt werden dürfen.
 *
 *  Diese Liste ist der Missbrauchsschutz: Wer keine fremden Ziele hinterlegen
 *  kann, kann den Endpunkt nicht als Phishing-Werkzeug verwenden. Genau das
 *  hält declarino.ch von den Sperrlisten fern, an denen die Fremddienste
 *  gescheitert sind. Sie begrenzt zugleich das Datenwachstum. */
const ALLOWED_TARGET_HOSTS = [
  "www.declarino.ch",
  "declarino.ch",
  "bar9.github.io", // Staging
];

/** Herkünfte, die den Endpunkt per Browser aufrufen dürfen.
 *  Staging und die lokale Entwicklung müssen mit, sonst ist der Teilen-Button
 *  ausserhalb der Produktion tot. */
const ALLOWED_ORIGINS = [
  "https://www.declarino.ch",
  "https://declarino.ch",
  "https://bar9.github.io",
];

/** Ob eine Origin erlaubt ist. localhost/127.0.0.1 mit beliebigem Port gilt
 *  als Entwicklung. */
export function allowedOrigin(origin) {
  if (!origin) return null;
  if (ALLOWED_ORIGINS.includes(origin)) return origin;
  if (/^https?:\/\/(localhost|127\.0\.0\.1)(:\d+)?$/.test(origin)) return origin;
  return null;
}

/** CORS-Header setzen. Ohne diese ist der Endpunkt aus dem WASM-Frontend
 *  nicht aufrufbar — daran sind cleanuri und ulvis gescheitert. */
export function applyCors(req, res) {
  const origin = allowedOrigin(req.headers.origin);
  if (origin) {
    res.setHeader("Access-Control-Allow-Origin", origin);
    // Antwort hängt von der Origin ab; ohne Vary könnte ein CDN die
    // Freigabe für eine fremde Herkunft ausliefern.
    res.setHeader("Vary", "Origin");
  }
  res.setHeader("Access-Control-Allow-Methods", "POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");
  return origin;
}

/** Prüft, ob `url` auf eine erlaubte Declarino-Adresse zeigt. */
export function isAllowedTarget(url) {
  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }
  if (parsed.protocol !== "https:" && parsed.protocol !== "http:") return false;
  return ALLOWED_TARGET_HOSTS.includes(parsed.hostname);
}

/** Base62-Alphabet: Kurz-Codes bleiben so per Doppelklick markierbar und
 *  lassen sich am Telefon vorlesen. */
const BASE62 = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/** Code für eine URL: die ersten Bits ihres SHA-256 in Base62.
 *
 *  Bewusst deterministisch statt zufällig: Wer dasselbe Rezept zweimal teilt,
 *  bekommt denselben Link, statt einen zweiten Eintrag anzulegen. Das hält
 *  den Datenbestand klein und macht die Funktion wiederholbar.
 *
 *  7 Zeichen sind rund 3.5e12 Möglichkeiten. Kollisionen werden beim
 *  Schreiben trotzdem erkannt (siehe shorten.js), nicht bloss angenommen. */
export function codeForUrl(url, length = 7) {
  const digest = createHash("sha256").update(url).digest();
  let value = 0n;
  for (const byte of digest.subarray(0, 8)) value = (value << 8n) | BigInt(byte);
  let code = "";
  for (let i = 0; i < length; i++) {
    code = BASE62[Number(value % 62n)] + code;
    value /= 62n;
  }
  return code;
}

// Speicherzugriff liegt in _storage.mjs (Turso oder Upstash, je nach
// gesetzten Umgebungsvariablen) und wird hier nur weitergereicht, damit die
// Funktionen einen einzigen Import haben.
export { storeIfAbsent, lookup, storageBackend } from "./_storage.mjs";
