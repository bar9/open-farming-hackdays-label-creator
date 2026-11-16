#!/usr/bin/env bash
# Routing-Test für ops/caddy/Caddyfile.
#
# Startet Caddy mit genau der Produktionsdatei, nur ohne TLS auf Testports,
# davor den echten Kurz-Link-Dienst mit einer Wegwerf-Datenbank, und prüft:
#   * /s/<code> liefert die 301-Weiterleitung, nicht die App (gedruckte
#     Etiketten wären sonst still tot)
#   * /api/shorten erreicht den Dienst
#   * SPA-Pfade liefern index.html mit no-cache
#   * /assets/* ist lange cachebar, Sicherheits-Header sind gesetzt
#   * der Apex leitet dauerhaft auf www um
#
# Aufruf: ops/caddy/test-routing.sh <pfad/zum/shortener-binary>
# Braucht: caddy, curl.
# Kein -e: jede Prüfung soll laufen und gezählt werden, auch nach einem Fehler.
set -uo pipefail

BIN=${1:?Aufruf: $0 <shortener-binary>}
HERE=$(cd "$(dirname "$0")" && pwd)
WORK=$(mktemp -d)
WWW_PORT=18080
APEX_PORT=18081
API_PORT=18082
CHECK_PORT=18083
PIDS=()

aufraeumen() {
  for pid in "${PIDS[@]}"; do kill "$pid" 2>/dev/null || true; done
  rm -rf "$WORK"
}
trap aufraeumen EXIT

# Testseite: eine index.html und eine gehashte Datei wie aus dx. index.html
# ist absichtlich grösser als 512 Bytes, darunter komprimiert Caddy nicht.
mkdir -p "$WORK/site/assets"
{ echo '<!doctype html><title>declarino-test</title>'; printf '<!-- %0600d -->\n' 0; } >"$WORK/site/index.html"
echo 'console.log(1)' >"$WORK/site/assets/app-dxh0123456789abcdef.js"

SHORTENER_DB="$WORK/links.db" SHORTENER_LISTEN="127.0.0.1:$API_PORT" "$BIN" serve &
PIDS+=($!)

DECLARINO_WWW="http://127.0.0.1:$WWW_PORT" \
DECLARINO_APEX="http://127.0.0.1:$APEX_PORT" \
DECLARINO_CHECK="127.0.0.1:$CHECK_PORT" \
DECLARINO_ROOT="$WORK/site" \
SHORTENER_UPSTREAM="127.0.0.1:$API_PORT" \
XDG_DATA_HOME="$WORK/caddy-data" XDG_CONFIG_HOME="$WORK/caddy-config" \
  caddy run --adapter caddyfile --config "$HERE/Caddyfile" >"$WORK/caddy.log" 2>&1 &
PIDS+=($!)

for _ in $(seq 50); do
  curl -fsS -o /dev/null "http://127.0.0.1:$WWW_PORT/" 2>/dev/null && break
  sleep 0.2
done

FEHLER=0
pruefe() { # name, bedingung (als Rückgabewert des vorigen Befehls)
  if [ "$2" = 0 ]; then echo "ok   $1"; else echo "FAIL $1"; FEHLER=$((FEHLER + 1)); fi
}
# Antwortkopf (ohne Body) für eine URL. Nur die Namen der Kopfzeilen werden
# kleingeschrieben, die Werte (z.B. Location mit Gross-/Kleinbuchstaben im
# Code) bleiben unverändert.
kopf() { curl -sS -o /dev/null -D - "$@" | tr -d '\r' | sed -E 's/^([A-Za-z0-9-]+):/\L\1:/; s/^HTTP/http/'; }

W="http://127.0.0.1:$WWW_PORT"

# Kurz-Link anlegen und auflösen, über Caddy.
ANTWORT=$(curl -sS -H 'Origin: https://www.declarino.ch' -H 'Content-Type: application/json' \
  -d '{"url":"https://www.declarino.ch/lebensmittelrecht?v=2&t=routing"}' "$W/api/shorten")
CODE=$(printf '%s' "$ANTWORT" | sed -n 's/.*"code":"\([0-9A-Za-z]*\)".*/\1/p')
[ -n "$CODE" ]; pruefe "POST /api/shorten über Caddy -> Code ($ANTWORT)" $?

H=$(kopf "$W/s/$CODE")
echo "$H" | grep -q '^http/1.1 301'; pruefe "/s/$CODE -> 301" $?
echo "$H" | grep -q '^location: https://www.declarino.ch/lebensmittelrecht?v=2&t=routing$'; pruefe "/s/$CODE -> Location stimmt" $?
echo "$H" | grep -q '^strict-transport-security: max-age=31536000'; pruefe "/s/ trägt HSTS" $?

H=$(kopf "$W/s/UnbekanntX")
echo "$H" | grep -q '^http/1.1 404'; pruefe "/s/<unbekannt> -> 404 (nicht die App)" $?

H=$(kopf "$W/api/redirect?code=$CODE")
echo "$H" | grep -q '^http/1.1 301'; pruefe "/api/redirect?code= (alter Pfad) -> 301" $?

for p in / /lebensmittelrecht /bio /knospe /favicon.ico; do
  H=$(kopf "$W$p")
  BODY=$(curl -sS "$W$p")
  { echo "$H" | grep -q '^http/1.1 200' && [[ "$BODY" == *declarino-test* ]]; }; pruefe "$p -> index.html" $?
  echo "$H" | grep -q '^cache-control: no-cache'; pruefe "$p -> no-cache" $?
done

H=$(kopf "$W/assets/app-dxh0123456789abcdef.js")
echo "$H" | grep -q '^cache-control: public, max-age=31536000, immutable'; pruefe "/assets/* -> immutable" $?
echo "$H" | grep -q '^x-content-type-options: nosniff'; pruefe "nosniff gesetzt" $?
echo "$H" | grep -q '^referrer-policy: strict-origin-when-cross-origin'; pruefe "Referrer-Policy gesetzt" $?
echo "$H" | grep -q '^server:'; [ $? != 0 ]; pruefe "kein Server-Header" $?

H=$(kopf -H 'Accept-Encoding: gzip' "$W/")
echo "$H" | grep -q '^content-encoding: gzip'; pruefe "Kompression aktiv" $?

H=$(kopf "http://127.0.0.1:$APEX_PORT/s/$CODE?x=1")
echo "$H" | grep -q '^http/1.1 301'; pruefe "Apex -> dauerhafte Umleitung (301)" $?
echo "$H" | grep -q "^location: https://www.declarino.ch/s/$CODE?x=1$"; pruefe "Apex -> https://www.declarino.ch{uri}" $?

# Prüfadresse (lokal, für den Smoke-Test im Deploy): gleiche Regeln wie www.
C="http://127.0.0.1:$CHECK_PORT"
H=$(kopf "$C/s/$CODE")
echo "$H" | grep -q '^http/1.1 301'; pruefe "Prüfadresse: /s/ -> 301" $?
H=$(kopf "$C/bio")
echo "$H" | grep -q '^cache-control: no-cache'; pruefe "Prüfadresse: SPA-Pfad -> index.html, no-cache" $?

if [ "$FEHLER" -ne 0 ]; then
  echo; echo "Caddy-Log:"; tail -20 "$WORK/caddy.log"
  echo "$FEHLER Prüfung(en) fehlgeschlagen."; exit 1
fi
echo; echo "Alle Routing-Prüfungen bestanden."
