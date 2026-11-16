# Kurz-Link-Dienst (`declarino.ch/s/…`)

Eigener URL-Shortener als kleines Rust-Programm (axum + SQLite), weil die
Fremddienste für Declarino unbrauchbar geworden sind (Begründung und Messwerte
im Anhang). Läuft auf dem eigenen Server hinter Caddy; Einrichtung und Betrieb
in [`ops/README.md`](../ops/README.md).

```
POST /api/shorten   { "url": "https://www.declarino.ch/..." }
                 -> { "code": "AbC1234",
                      "short_url": "https://www.declarino.ch/s/AbC1234" }

GET  /s/:code    -> 301 auf die hinterlegte Adresse
```

`GET /api/redirect?code=…` antwortet gleich wie `/s/:code`; der Pfad war auf
der früheren Plattform öffentlich erreichbar und bleibt deshalb bestehen.

| Datei | Zweck |
|---|---|
| `src/rules.rs` | Allowlists, CORS-Herkünfte, Code-Erzeugung |
| `src/storage.rs` | SQLite-Datei: Schema, Speichern, Nachschlagen, Import |
| `src/lib.rs` | HTTP-Endpunkte |
| `src/main.rs` | Dienst starten, `import`/`export`/`verify` |
| `tests/http.rs` | HTTP-Tests gegen den echten Router |
| `uitest_share_modal.mjs` | Prüft den Teilen-Dialog im Browser |

Antwortformat und Feldname `short_url` sind von spoo.me übernommen, damit die
Rust-Seite (`src/services/url_shortener.rs`) denselben Parser nutzt.

## Entwurfsentscheidungen

**Codes sind Hashes, keine Zufallswerte.** Der Code sind die ersten 64 Bit von
SHA-256 der Ziel-URL in Base62. Dasselbe Rezept ergibt damit immer denselben
Kurz-Link, wiederholtes Teilen erzeugt keine Karteileichen. Bei einer Kollision
(gleicher Code, anderes Ziel) wächst der Code, statt den fremden Eintrag zu
überschreiben, sonst landete ein bereits verschickter Link stillschweigend
woanders. Die Berechnung ist bitgenau dieselbe wie in der früheren
JavaScript-Fassung; `rules.rs` hält das mit Testwerten fest.

**Kein Ablaufdatum.** Kurz-Links landen auf gedruckten Etiketten und dürfen
nicht verschwinden.

**Nur Declarino-Adressen.** Die Allowlist in `rules.rs` ist der
Missbrauchsschutz: Der Endpunkt taugt nicht als Phishing-Werkzeug und bleibt
deshalb von den Sperrlisten fern, an denen die Fremddienste scheitern.

**Grenzen und Fehler** sind unverändert übernommen: URL höchstens 16 KiB
(413 `URL zu lang`), fremdes Ziel 400 `Nur declarino.ch-Adressen`, fremde
Origin 403 `Herkunft nicht erlaubt`, ungültiges JSON 400, fehlendes Feld 400
`Feld 'url' fehlt`, unbekannter Code 404 `Unbekannter Link`.

## Speicher

Eine SQLite-Datei auf dem Server, `/var/lib/shortener/links.db` (systemd
`StateDirectory=shortener`), im WAL-Modus. Schema:

```sql
CREATE TABLE links (
  code TEXT PRIMARY KEY,
  url TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
```

Der Dienst legt Datei und Tabelle beim ersten Start selbst an; ein frischer
Server braucht keinen Migrationsschritt.

## Entwickeln und testen

```bash
cd api
cargo test                     # Regeln, Speicher und alle HTTP-Fälle
cargo clippy --all-targets -- -D warnings
SHORTENER_DB=/tmp/links.db cargo run    # lokal auf 127.0.0.1:3000
```

Die HTTP-Tests sind aus dem früheren Selbsttest portiert: Kürzen,
301-Auflösung, Wiederholbarkeit der Codes, Kollisionen, Ablehnung fremder Ziele
und Herkünfte, Preflight, unbekannte Codes, Grössenlimits, Auflistbarkeit.

Das Zusammenspiel mit Caddy (der Kurz-Link darf nie vom SPA-Fallback
verschluckt werden) prüft `ops/caddy/test-routing.sh` mit der echten
Caddy-Konfiguration; beides läuft in CI.

## Sicherung

Kurz-Links sind dauerhafter Zustand: Einer auf einem gedruckten Etikett lässt
sich nicht mehr ändern. Geht die Datenbank verloren, sind alle ausgedruckten
Etiketten unbrauchbar. Der Bestand ist winzig (~1 KB je Eintrag).

* Nächtlich `sqlite3 .backup` per systemd-Timer nach
  `/var/backups/shortener/`, die letzten 14 Kopien bleiben (siehe
  `ops/shortener/`).
* Von Hand, auf dem Server:

  ```bash
  sudo -u shortener /opt/shortener/shortener export > links.json
  sudo -u shortener /opt/shortener/shortener import links.json   # idempotent
  sudo -u shortener /opt/shortener/shortener verify links.json
  ```

`import` legt nur fehlende Codes an, ändert nie einen bestehenden und meldet
einen Code mit abweichender URL als Konflikt (Exit-Code 1). Er wendet dieselbe
Allowlist an wie der Endpunkt: eine manipulierte Sicherungsdatei kann keine
fremden Ziele einschleusen.

`ops/migration/verify-live.sh links.json` prüft jeden Code gegen die
öffentliche Seite und belegt damit, was ein Empfänger tatsächlich erlebt.

Exportdateien enthalten Rezepturen samt Produzentendaten und gehören nicht ins
Repo (`.gitignore`).

## Oberfläche

Der Teilen-Dialog kürzt standardmässig und ohne Zusatzklick: Kurz-Link ist
vorausgewählt, das Kürzen startet beim Öffnen, der vollständige Link bleibt als
zweite Option daneben.

Der Hinweistext richtet sich nach dem tatsächlichen Anbieter: Kommt der Link von
declarino.ch, steht das da; springt die Rückfallebene ein, wird der
Fremdanbieter genannt. Scheitert das Kürzen, schaltet der Dialog auf den
vollständigen Link um und zeigt einen Hinweis, statt ein leeres Feld stehen zu
lassen.

Geprüft mit `uitest_share_modal.mjs` (WebDriver, siehe Kopf der Datei).

## Rückfallebene

Fällt der Endpunkt aus, versucht das Frontend der Reihe nach spoo.me, tinyurl
und da.gd (Details in `src/services/url_shortener.rs`). Der Teilen-Button ist
also nie ganz tot, liefert dann aber wieder Links mit den unten beschriebenen
Nachteilen.

---

## Anhang: Warum ein eigener Dienst

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
gemeinsamer Infrastruktur aus. Tatsächlich sind es unabhängige Anbieter (OVH
bzw. Cloudflare), die nur beide umgeleitet werden; mit umgangenem DNS
antworten beide normal.

`declarino.ch` steht auf keiner Shortener-Sperrliste, weil die Domain eine
eigene Reputation hat.
