// POST /api/shorten  { "url": "https://www.declarino.ch/..." }
//   -> 200 { "short_url": "https://www.declarino.ch/s/AbC1234", "code": "..." }
//
// Antwortet bewusst im selben JSON-Format wie spoo.me (`short_url`), damit die
// Rust-Seite denselben Parser verwenden kann.

import {
  SHORT_BASE,
  applyCors,
  codeForUrl,
  isAllowedTarget,
  lookup,
  storeIfAbsent,
} from "./_lib.mjs";

/** Obergrenze für die Ziel-URL. Ein volles Rezept liegt bei ~4.5 KB; 16 KB
 *  lassen viel Luft und verhindern trotzdem, dass jemand den Speicher mit
 *  Riesen-URLs füllt. */
const MAX_URL_LENGTH = 16 * 1024;

export default async function handler(req, res) {
  const origin = applyCors(req, res);

  // Preflight: der Browser fragt vor dem eigentlichen POST an.
  if (req.method === "OPTIONS") return res.status(204).end();
  if (req.method !== "POST") {
    return res.status(405).json({ error: "Nur POST" });
  }
  // Ohne erlaubte Origin gar nicht erst arbeiten. Direkte Aufrufe ohne
  // Origin-Header (curl, Server) sind erlaubt: CORS schützt Browser-Nutzer,
  // nicht den Endpunkt selbst.
  if (req.headers.origin && !origin) {
    return res.status(403).json({ error: "Herkunft nicht erlaubt" });
  }

  // Vercel parst JSON-Bodies selbst, lässt bei anderem Content-Type aber einen
  // String durch.
  let body = req.body;
  if (typeof body === "string") {
    try {
      body = JSON.parse(body);
    } catch {
      return res.status(400).json({ error: "Ungültiges JSON" });
    }
  }
  const url = body?.url;
  if (typeof url !== "string" || url.length === 0) {
    return res.status(400).json({ error: "Feld 'url' fehlt" });
  }
  if (url.length > MAX_URL_LENGTH) {
    return res.status(413).json({ error: "URL zu lang" });
  }
  if (!isAllowedTarget(url)) {
    // Kein fremdes Ziel: siehe ALLOWED_TARGET_HOSTS in _lib.mjs.
    return res.status(400).json({ error: "Nur declarino.ch-Adressen" });
  }

  try {
    // Der Code ist der Hash der URL, dieselbe URL ergibt also denselben Link.
    // Bei einer Kollision (anderes Ziel, gleicher Code) wird der Code
    // schrittweise verlängert, statt den fremden Eintrag zu überschreiben —
    // sonst würde ein bereits geteilter Link stillschweigend woanders landen.
    for (let length = 7; length <= 12; length++) {
      const code = codeForUrl(url, length);
      if (await storeIfAbsent(code, url)) {
        return res.status(200).json({ code, short_url: `${SHORT_BASE}/s/${code}` });
      }
      const existing = await lookup(code);
      if (existing === url) {
        // Schon vorhanden: derselbe Link, kein neuer Eintrag.
        return res.status(200).json({ code, short_url: `${SHORT_BASE}/s/${code}` });
      }
    }
    return res.status(500).json({ error: "Kein freier Code gefunden" });
  } catch (error) {
    console.error("shorten failed:", error);
    return res.status(500).json({ error: "Speichern fehlgeschlagen" });
  }
}
