// Prüft die Rewrite-Regeln aus vercel.json mit @vercel/routing-utils —
// derselben Bibliothek, die Vercel selbst zum Übersetzen von vercel.json in
// Routing-Regeln benutzt. Damit ist das Verhalten belegbar, ohne die Regeln
// erst in der Produktion auszuprobieren.
//
// Der Anlass: ein fehlerhafter SPA-Catch-all kann alle Kurz-Links stumm
// zerstören. Ein Rewrite `/(.*)` ohne Ausnahmen liefert für /s/<code> die
// App mit HTTP 200 statt der 301-Weiterleitung — gedruckte Etiketten wären
// dann tot, ohne dass es jemand bemerkt. Dieser Test hält die Ausnahmen fest.
//
// Zwei Dinge halten die Kurz-Links am Leben, und beide sind nötig:
//   1. der negative Lookahead `(?!…s/…)` in der SPA-Regel
//   2. die Position der `/s/`-Regel VOR der SPA-Regel
// Je einzeln entfernt bleibt der Test grün (das jeweils andere fängt es ab);
// fallen beide weg, schlägt er fehl. Wer eine der beiden ändert, sollte die
// andere prüfen.
//
// Aufruf:  node api/routecheck.cjs vercel.json api/routecases.json
const { getTransformedRoutes, normalizeRoutes } = require("@vercel/routing-utils");
const fs = require("fs");

const config = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const transformed = getTransformedRoutes({
  rewrites: config.rewrites,
  redirects: config.redirects,
  headers: config.headers,
  cleanUrls: config.cleanUrls,
  trailingSlash: config.trailingSlash,
});
if (transformed.error) {
  console.error("vercel.json ungültig:", JSON.stringify(transformed.error, null, 2));
  process.exit(1);
}
const normalized = normalizeRoutes(transformed.routes);
if (normalized.error) {
  console.error("Routen ungültig:", JSON.stringify(normalized.error, null, 2));
  process.exit(1);
}
const routes = normalized.routes ?? [];

// Statische Dateien, die im Deployment wirklich existieren. Vercel prüft das
// Dateisystem vor den Rewrites ("filesystem"-Phase), deshalb dürfen Rewrites
// diese Pfade gar nicht erst sehen.
// Genau die Dateien, die im echten Deployment existieren (bar9/declarino).
const STATIC = new Set([
  "/index.html",
  "/404.html",
  "/CNAME",
  "/assets/open-farming-hackdays-label-creator_bg-dxh2e22bebfc77d5b5c.wasm",
  "/assets/open-farming-hackdays-label-creator-dxhc53cec43373bdb6.js",
  "/assets/tailwind-dxhb2187c5f8e774034.css",
  "/assets/favicon-dxhd59db873b822e8df.svg",
]);
// Serverless-Funktionen aus api/. Vercel löst sie ebenfalls in der
// filesystem-Phase auf, also vor den Rewrites — deshalb gehören sie hier
// dazu, sonst prüft der Test etwas anderes als die Plattform tut.
for (const f of ["shorten", "redirect"]) STATIC.add(`/api/${f}`);

/** Bildet nach, was Vercel mit einem Pfad macht. */
function resolve(pathname) {
  if (STATIC.has(pathname)) return { dest: pathname, via: "filesystem" };
  for (const route of routes) {
    if (!route.src) continue;
    const re = new RegExp(route.src);
    const m = pathname.match(re);
    if (!m) continue;
    let dest = route.dest ?? pathname;
    // $1, $2 … durch die Fundstellen ersetzen, benannte Gruppen ebenso.
    dest = dest.replace(/\$(\d+)/g, (_, n) => m[Number(n)] ?? "");
    dest = dest.replace(/\$([a-zA-Z_][a-zA-Z0-9_]*)/g, (_, n) => m.groups?.[n] ?? "");
    return { dest, via: route.src };
  }
  return { dest: null, via: "kein Treffer -> 404" };
}

const cases = JSON.parse(fs.readFileSync(process.argv[3], "utf8"));
let failures = 0;
for (const c of cases) {
  const got = resolve(c.path);
  const ok = c.expect === null ? got.dest === null : got.dest === c.expect;
  if (!ok) failures++;
  console.log(
    `${ok ? "ok  " : "FAIL"} ${c.path.padEnd(46)} -> ${String(got.dest)}` +
      (ok ? "" : `   (erwartet: ${c.expect})`)
  );
}
console.log(failures === 0 ? "\nAlle Routing-Fälle korrekt." : `\n${failures} Fall/Fälle falsch.`);
process.exit(failures === 0 ? 0 : 1);
