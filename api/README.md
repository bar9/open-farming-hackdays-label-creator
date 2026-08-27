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

## Stand

In Produktion verifiziert am 2026-08-27 gegen `declarino.ch` mit Turso als
Speicher: ein vollständiges Rezept (4444 Zeichen) ergibt einen 34 Zeichen
langen Kurz-Link, der mit HTTP 301 und **0 Bytes Body** auf die
zeichengenau identische Original-URL weiterleitet — also ohne
Zwischenseite. Weiterleitung ~75 ms, Kürzen ~600 ms.

## Aufbau

```
POST /api/shorten   { "url": "https://www.declarino.ch/..." }
                 -> { "code": "AbC1234",
                      "short_url": "https://www.declarino.ch/s/AbC1234" }

GET  /s/:code    -> 301 auf die hinterlegte Adresse
```

| Datei | Zweck |
|---|---|
| `api/_lib.mjs` | Allowlists, Code-Erzeugung |
| `api/_storage.mjs` | Speicherzugriff, Turso **oder** Upstash |
| `api/shorten.mjs` | Kürzen (POST) |
| `api/redirect.mjs` | Auflösen (301), erreichbar über den Rewrite in `vercel.json` |
| `api/selftest.mjs` | End-to-End-Test gegen eine echte Datenbank |
| `api/backup.mjs` | Export, Import und Prüfung aller Kurz-Links |

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

## Speicher

Vercel-Funktionen haben ein read-only Dateisystem und leben nur Millisekunden;
eine lokale SQLite-Datei ist deshalb unmöglich (sie wäre beim nächsten Aufruf
weg). Beide unterstützten Anbieter sprechen daher **HTTPS** statt eines
Verbindungsprotokolls:

| Anbieter | Erkennungsvariable | Modell |
|---|---|---|
| **Turso** (libSQL) | `TURSO_DATABASE_URL` + `TURSO_AUTH_TOKEN` | SQLite über HTTPS |
| **Upstash** (Redis) | `KV_REST_API_URL` + `KV_REST_API_TOKEN` | Key/Value über HTTPS |

Die Wahl trifft allein die Umgebung, im Code steht keine Festlegung (siehe
`_storage.mjs`). Sind beide gesetzt, gewinnt Turso. Ein Anbieterwechsel ist
damit eine Frage der Projekteinstellungen — wichtig, weil Kurz-Links auf
gedruckten Etiketten landen und den Anbieter überleben müssen.

Bei Turso legt der erste Schreibzugriff die Tabelle selbst an; eine frisch
bereitgestellte Datenbank ist ohne manuellen Schritt benutzbar.

## Einrichtung

Einmalig für das Vercel-Projekt hinter `declarino.ch`:

1. **Datenbank verbinden** — über den Vercel Marketplace, es braucht also
   kein separates Konto beim Anbieter:

   ```
   vercel install turso      # oder: vercel install upstash
   ```

   Alternativ im Dashboard unter *Storage → Marketplace*. Die Integration
   setzt die Umgebungsvariablen selbst.

2. **Neu deployen.** Umgebungsvariablen greifen erst beim nächsten
   Deployment; blosses Speichern reicht nicht.

Ohne Konfiguration antwortet `/api/shorten` mit HTTP 500, und das Frontend
fällt still auf die Fremddienste zurück — der Teilen-Button bleibt also
funktionsfähig.

Die Zugangsdaten liegen ausschliesslich serverseitig. Im WASM-Frontend wäre
jedes Geheimnis auslesbar (`strings …wasm | grep`).

## Testen

```bash
TURSO_DATABASE_URL=… TURSO_AUTH_TOKEN=… node api/selftest.mjs
KV_REST_API_URL=…    KV_REST_API_TOKEN=… node api/selftest.mjs
```

Prüft gegen eine echte Redis-Instanz: Kürzen, 301-Auflösung, Wiederholbarkeit
der Codes, Ablehnung fremder Ziele und Herkünfte, Preflight, unbekannte Codes,
Grössenlimit.

Testdatenbanken ohne Anmeldung:

```bash
# Turso: echter libSQL-Server lokal, gleiche HTTP-API wie Turso Cloud
docker run -d -p 8090:8080 -e SQLD_NODE=primary \
  ghcr.io/tursodatabase/libsql-server:latest
TURSO_DATABASE_URL=http://localhost:8090 TURSO_AUTH_TOKEN=dummy \
  node api/selftest.mjs

# Upstash: Wegwerf-Datenbank, gültig 3 Tage
curl -X POST https://upstash.com/start-redis
```

## Sicherung

Kurz-Links sind dauerhafter Zustand: Einer auf einem gedruckten Etikett lässt
sich nicht mehr ändern. Geht die Datenbank verloren, sind alle ausgedruckten
Etiketten unbrauchbar — die lange URL trägt ihre Daten selbst und ist davon
nicht betroffen, der Kurz-Link nicht.

Der Bestand ist winzig (~1 KB je Eintrag), ein vollständiger Export kostet
also fast nichts:

```bash
TURSO_DATABASE_URL=… TURSO_AUTH_TOKEN=… node api/backup.mjs export > links.json
node api/backup.mjs import links.json    # nach Verlust oder Anbieterwechsel
node api/backup.mjs verify links.json    # prüft gegen die Live-Seite
```

`verify` fragt `declarino.ch/s/<code>` ab statt die Datenbank und belegt
damit, was ein Empfänger tatsächlich erlebt. Exit-Code 1 bei Fehlern, also
für Überwachung geeignet.

**Anbieterwechsel** funktioniert damit ohne Linkverlust: exportieren, die
Umgebungsvariablen umstellen, importieren. Die Codes bleiben gleich, weil sie
aus der URL abgeleitet sind. Verifiziert für Turso → Upstash.

Der Import wendet dieselbe Allowlist an wie der Endpunkt: eine manipulierte
Sicherungsdatei kann keine fremden Ziele einschleusen.

## Deployment

`declarino.ch` wird von Vercel aus dem Repo `bar9/declarino` (Branch
`gh-pages`) bedient. Der Workflow `deploy-production.yml` publiziert dorthin
und ersetzt den Inhalt dabei vollständig (`keep_files` ist standardmässig
`false`). `api/` und `vercel.json` werden deshalb bei jedem Lauf aus diesem
Repo mitkopiert — lägen sie nur im Zielrepo, wären sie nach dem nächsten
Deploy weg.

Der Workflow übergibt zusätzlich `clean: true`. Diese Option kennt
`peaceiris/actions-gh-pages` nicht; sie wird mit einer Warnung ignoriert und
hat keine Wirkung. Das Ersetzen passiert ohnehin über `keep_files: false`.

## Rückfallebene

Fällt der Endpunkt aus, versucht das Frontend der Reihe nach spoo.me,
tinyurl und da.gd (Details in `src/services/url_shortener.rs`). Der
Teilen-Button ist also nie ganz tot, liefert dann aber wieder Links mit den
oben beschriebenen Nachteilen.

## Offener Nebenbefund: Deep-Links liefern HTTP 404

Nicht Teil des Kurz-Link-Dienstes, aber beim Testen aufgefallen und bewusst
**nicht** mitgeändert:

```
curl -o /dev/null -w '%{http_code}' 'https://www.declarino.ch/lebensmittelrecht?v=2'
404
```

Die App lädt trotzdem, weil `404.html` eine Kopie von `index.html` ist. Der
Status ist aber formal falsch. Betrifft auch Staging, besteht also seit
jeher und unabhängig vom Shortener. Ein Kurz-Link erbt das Verhalten, weil
er auf genau solche Deep-Links zeigt.

Folgen: Suchmaschinen indexieren die Seiten nicht, Link-Vorschauen (z.B. in
Messengern) können fehlschlagen, Fehler-Monitoring meldet Rauschen.

Behebbar wäre es mit einem SPA-Fallback-Rewrite in `vercel.json`, der nach
der `/s/:code`-Regel stehen müsste, damit er die Kurz-Links nicht
verschluckt. Ein fehlerhafter Rewrite legt allerdings die ganze Seite lahm,
deshalb gehört das zuerst auf ein Preview-Deployment.
