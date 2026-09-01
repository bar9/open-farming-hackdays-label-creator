// GET /s/:code -> 301 auf die hinterlegte Declarino-Adresse.
//
// Wird über den Rewrite in vercel.json erreicht. Ein echter 301 ohne
// Zwischenseite ist der ganze Zweck der Übung: Fremddienste blenden für
// frische Links Warnseiten ein.

import { lookup } from "./_lib.mjs";

/** Erlaubte Codeform. Grosszügiger als die vergebenen 7 bis 12 Zeichen, damit
 *  eine spätere Längenänderung alte Links nicht ungültig macht; alles andere
 *  hält Unfug wie Pfad-Traversal aus dem Datenbankschlüssel heraus. */
const CODE_PATTERN = /^[0-9a-zA-Z]{4,16}$/;

export default async function handler(req, res) {
  const code = (req.query?.code ?? "").toString();
  if (!CODE_PATTERN.test(code)) {
    return res.status(404).send("Unbekannter Link");
  }

  let target;
  try {
    target = await lookup(code);
  } catch (error) {
    console.error("lookup failed:", error);
    return res.status(500).send("Nachschlagen fehlgeschlagen");
  }

  if (!target) {
    return res.status(404).send("Unbekannter Link");
  }

  // 301 statt 302: Kurz-Links sind unveränderlich (der Code ist der Hash des
  // Ziels), also darf der Browser das Ergebnis behalten.
  res.setHeader("Location", target);
  res.setHeader("Cache-Control", "public, max-age=86400");
  return res.status(301).end();
}
