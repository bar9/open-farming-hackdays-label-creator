// End-to-End-Test der Shortener-Funktionen gegen eine echte Redis-Instanz.
//
// Läuft ohne Vercel-CLI: die Handler sind gewöhnliche (req, res)-Funktionen,
// also genügt ein Minimal-Double für `res`. Getestet wird damit genau der
// Code, der später deployt wird — inklusive Redis-Zugriff, CORS und Rewrite-
// Parameter.
//
// Läuft gegen den Anbieter, dessen Variablen gesetzt sind:
//   TURSO_DATABASE_URL=... TURSO_AUTH_TOKEN=...   node api/selftest.mjs
//   KV_REST_API_URL=...    KV_REST_API_TOKEN=...  node api/selftest.mjs

import shorten from "./shorten.mjs";
import redirect from "./redirect.mjs";
import { codeForUrl, storageBackend } from "./_lib.mjs";

const backend = storageBackend();
if (!backend) {
  console.error(
    "Kein Speicher konfiguriert. Setze TURSO_DATABASE_URL/TURSO_AUTH_TOKEN\n" +
      "oder KV_REST_API_URL/KV_REST_API_TOKEN."
  );
  process.exit(2);
}
console.log(`Speicher: ${backend}\n`);

let failures = 0;
function check(name, condition, detail = "") {
  const status = condition ? "ok  " : "FAIL";
  if (!condition) failures++;
  console.log(`${status} ${name}${detail && !condition ? ` -> ${detail}` : ""}`);
}

/** Minimales `res`-Double mit der von Vercel benutzten Fluent-API. */
function makeRes() {
  const res = {
    statusCode: null,
    headers: {},
    body: undefined,
    setHeader(k, v) {
      this.headers[k.toLowerCase()] = v;
      return this;
    },
    status(code) {
      this.statusCode = code;
      return this;
    },
    json(payload) {
      this.body = payload;
      return this;
    },
    send(payload) {
      this.body = payload;
      return this;
    },
    end() {
      return this;
    },
  };
  return res;
}

const ORIGIN = "https://www.declarino.ch";
const call = async (handler, req) => {
  const res = makeRes();
  await handler({ headers: {}, query: {}, ...req }, res);
  return res;
};

const RECIPE = `${ORIGIN}/lebensmittelrecht?v=2&product_title=Brownies&ingredients[0][name]=Butter&t=${Date.now()}`;

// 1. Der Normalfall: kürzen und wieder auflösen.
const created = await call(shorten, {
  method: "POST",
  headers: { origin: ORIGIN },
  body: { url: RECIPE },
});
check("shorten antwortet 200", created.statusCode === 200, JSON.stringify(created.body));
check(
  "short_url zeigt auf declarino.ch/s/",
  created.body?.short_url?.startsWith("https://www.declarino.ch/s/"),
  created.body?.short_url
);
check(
  "CORS-Header für erlaubte Origin",
  created.headers["access-control-allow-origin"] === ORIGIN
);
check("Vary: Origin gesetzt", created.headers["vary"] === "Origin");

const code = created.body?.code;
check("Code ist 7 Zeichen Base62", /^[0-9a-zA-Z]{7}$/.test(code ?? ""), code);

// 2. Der Redirect muss ohne Zwischenseite auf das Original zeigen.
const hop = await call(redirect, { method: "GET", query: { code } });
check("redirect antwortet 301", hop.statusCode === 301, String(hop.statusCode));
check("Location ist die Original-URL", hop.headers["location"] === RECIPE, hop.headers["location"]);

// 3. Zweimal Teilen darf keinen zweiten Eintrag erzeugen.
const again = await call(shorten, {
  method: "POST",
  headers: { origin: ORIGIN },
  body: { url: RECIPE },
});
check("gleiche URL -> gleicher Code", again.body?.code === code, `${again.body?.code} vs ${code}`);

// 4. Fremde Ziele müssen abgelehnt werden (Missbrauchsschutz).
for (const bad of [
  "https://example.com/phishing",
  "https://www.declarino.ch.evil.com/x",
  "javascript:alert(1)",
  "notaurl",
]) {
  const res = await call(shorten, {
    method: "POST",
    headers: { origin: ORIGIN },
    body: { url: bad },
  });
  check(`fremdes Ziel abgelehnt: ${bad.slice(0, 40)}`, res.statusCode === 400, String(res.statusCode));
}

// 5. Staging und localhost müssen den Endpunkt benutzen dürfen.
for (const origin of ["https://bar9.github.io", "http://localhost:8080"]) {
  const res = await call(shorten, {
    method: "POST",
    headers: { origin },
    body: { url: `${ORIGIN}/lebensmittelrecht?from=${encodeURIComponent(origin)}` },
  });
  check(`Origin erlaubt: ${origin}`, res.statusCode === 200 && res.headers["access-control-allow-origin"] === origin);
}

// 6. Fremde Herkunft darf nicht durchkommen.
const evil = await call(shorten, {
  method: "POST",
  headers: { origin: "https://evil.example" },
  body: { url: RECIPE },
});
check("fremde Origin abgelehnt", evil.statusCode === 403, String(evil.statusCode));

// 7. Staging-URLs sind erlaubte Ziele (Kurz-Link zeigt dann dorthin zurück).
const staging = await call(shorten, {
  method: "POST",
  headers: { origin: "https://bar9.github.io" },
  body: { url: "https://bar9.github.io/open-farming-hackdays-label-creator/?v=2" },
});
check("Staging-Ziel erlaubt", staging.statusCode === 200, JSON.stringify(staging.body));

// 8. Preflight muss ohne Body durchgehen.
const preflight = await call(shorten, { method: "OPTIONS", headers: { origin: ORIGIN } });
check("OPTIONS -> 204", preflight.statusCode === 204);

// 9. Unbekannte und unsinnige Codes: 404 statt Fehler.
for (const bad of ["nichtvorhanden", "../etc/passwd", "", "zz"]) {
  const res = await call(redirect, { method: "GET", query: { code: bad } });
  check(`unbekannter Code -> 404: ${JSON.stringify(bad)}`, res.statusCode === 404, String(res.statusCode));
}

// 10. Zu lange URLs abweisen, damit der Speicher nicht vollläuft.
const huge = await call(shorten, {
  method: "POST",
  headers: { origin: ORIGIN },
  body: { url: `${ORIGIN}/lebensmittelrecht?x=${"a".repeat(17000)}` },
});
check("übergrosse URL -> 413", huge.statusCode === 413, String(huge.statusCode));

// 11. Der Code muss deterministisch aus der URL folgen (reine Funktion).
check(
  "codeForUrl ist deterministisch",
  codeForUrl(RECIPE) === codeForUrl(RECIPE) && codeForUrl(RECIPE) === code
);
check("verschiedene URLs -> verschiedene Codes", codeForUrl("a") !== codeForUrl("b"));

console.log(
  failures === 0
    ? `\nAlle Prüfungen bestanden (Speicher: ${backend}).`
    : `\n${failures} Prüfung(en) fehlgeschlagen (Speicher: ${backend}).`
);
process.exit(failures === 0 ? 0 : 1);
