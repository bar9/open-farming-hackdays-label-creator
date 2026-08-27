# Kurz-Link-Dienst (`declarino.ch/s/…`)

Eigener URL-Shortener als zwei Vercel-Funktionen. Ersetzt die Fremddienste,
die für Declarino unbrauchbar geworden sind.

## Warum eigener Dienst

Ein Dienst, der ohne Konto beliebige Ziele kürzt, ist genau das Werkzeug, mit
dem Phishing-Links gebaut werden. Provider sperren die Kategorie deshalb per
DNS. Gemessen am 2026-08-27 aus einem Swisscom-Anschluss:

| Dienst | Ergebnis |
|---|---|
| da.gd, spoo.me | DNS zeigt auf Swisscom-Sperrserver `195.186.4.x`, TLS-Handshake scheitert → im Browser `Failed to fetch` |
| is.gd, v.gd | HTTP 200 mit `Error, database insert failed` (gemeinsame Datenbank) |
| tinyurl | funktioniert, zeigt aber sporadisch Warn-/Preview-Zwischenseiten |
| ulvis, cleanuri, kurzelinks | kein CORS-Header → aus dem Browser nicht nutzbar |
| bitly, t.ly, short.io, cutt.ly | API-Key nötig; im WASM-Frontend wäre der auslesbar |

Dass da.gd und spoo.me *dieselbe* Sperr-IP bekommen, sah zunächst nach
gemeinsamer Infrastruktur aus. Tatsächlich sind es unabhängige Anbieter
(OVH bzw. Cloudflare), die nur beide umgeleitet werden — mit umgangenem DNS
antworten beide normal.

`declarino.ch` steht auf keiner Shortener-Sperrliste, weil die Domain eine
eigene Reputation hat. Der Endpunkt nimmt zudem nur Declarino-Adressen an und
taugt damit nicht als Phishing-Werkzeug.

## Aufbau

```
POST /api/shorten   { "url": "https://www.declarino.ch/..." }
                 -> { "code": "AbC1234",
                      "short_url": "https://www.declarino.ch/s/AbC1234" }

GET  /s/:code    -> 301 auf die hinterlegte Adresse
```

| Datei | Zweck |
|---|---|
| `api/_lib.mjs` | Allowlists, Code-Erzeugung, Redis-Zugriff |
| `api/shorten.mjs` | Kürzen (POST) |
| `api/redirect.mjs` | Auflösen (301), erreichbar über den Rewrite in `vercel.json` |
| `api/selftest.mjs` | End-to-End-Test gegen eine echte Redis-Instanz |

Antwortformat und Feldname `short_url` sind bewusst von spoo.me übernommen,
damit die Rust-Seite (`src/services/url_shortener.rs`) denselben Parser nutzt.

### Codes sind Hashes, keine Zufallswerte

Der Code sind die ersten Bits von SHA-256 der Ziel-URL in Base62. Dasselbe
Rezept ergibt damit immer denselben Kurz-Link, wiederholtes Teilen erzeugt
keine Karteileichen. Bei einer Kollision (gleicher Code, anderes Ziel) wird
der Code verlängert statt den fremden Eintrag zu überschreiben — sonst würde
ein bereits verschickter Link stillschweigend woanders landen.

Einträge haben **kein** Ablaufdatum: Kurz-Links landen auf gedruckten
Etiketten und dürfen nicht verschwinden.

## Einrichtung

Einmalig für das Vercel-Projekt hinter `declarino.ch`:

1. **Upstash Redis verbinden** (Gratis-Kontingent: 256 MB, 500'000
   Kommandos/Monat — ein Eintrag ist ~1 KB, das reicht für Zehntausende
   Rezepte):

   ```
   vercel install upstash
   ```

   Alternativ im Vercel-Dashboard unter *Storage → Marketplace → Upstash*.
   Die Integration setzt `KV_REST_API_URL` und `KV_REST_API_TOKEN` selbst.

2. **Prüfen**, dass die Variablen im Projekt gesetzt sind. Ohne sie
   antwortet `/api/shorten` mit HTTP 500, und das Frontend fällt still auf
   die Fremddienste zurück.

Die Zugangsdaten liegen ausschliesslich serverseitig. Im WASM-Frontend wäre
jedes Geheimnis auslesbar (`strings …wasm | grep`).

## Testen

```bash
KV_REST_API_URL=… KV_REST_API_TOKEN=… node api/selftest.mjs
```

Prüft gegen eine echte Redis-Instanz: Kürzen, 301-Auflösung, Wiederholbarkeit
der Codes, Ablehnung fremder Ziele und Herkünfte, Preflight, unbekannte Codes,
Grössenlimit.

Eine Wegwerf-Datenbank für Tests gibt es ohne Anmeldung:

```bash
curl -X POST https://upstash.com/start-redis   # gültig 3 Tage
```

## Deployment

`declarino.ch` wird von Vercel aus dem Repo `bar9/declarino` (Branch
`gh-pages`) gebaut, das der Workflow `deploy-production.yml` mit `clean: true`
überschreibt. `api/` und `vercel.json` werden deshalb bei jedem Lauf aus
diesem Repo mitkopiert — lägen sie nur im Zielrepo, wären sie nach dem
nächsten Deploy weg.

## Rückfallebene

Fällt der Endpunkt aus, versucht das Frontend der Reihe nach spoo.me,
tinyurl und da.gd (Details in `src/services/url_shortener.rs`). Der
Teilen-Button ist also nie ganz tot, liefert dann aber wieder Links mit den
oben beschriebenen Nachteilen.
