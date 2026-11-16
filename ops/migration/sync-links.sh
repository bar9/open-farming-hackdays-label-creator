#!/usr/bin/env bash
# VORÜBERGEHEND: gleicht die Kurz-Links aus Turso mit der SQLite-Datenbank auf
# dem Server ab. Beliebig oft ausführbar, z.B. jetzt und noch einmal direkt
# vor der DNS-Umstellung, damit inzwischen auf der alten Plattform erzeugte
# Links nicht verloren gehen.
#
# Idempotent und nicht zerstörend: `shortener import` legt nur fehlende Codes
# an, ändert nie einen bestehenden und meldet Abweichungen als Konflikt.
#
#   TURSO_DATABASE_URL=… TURSO_AUTH_TOKEN=… ops/migration/sync-links.sh
#
# Braucht lokal: node, ssh/scp mit Admin-Zugang zum Server.
#   SERVER  (Standard debian@83.228.242.209)
#   SSH_KEY (Standard ~/.ssh/infomaniak)
set -euo pipefail

SERVER=${SERVER:-debian@83.228.242.209}
SSH_KEY=${SSH_KEY:-$HOME/.ssh/infomaniak}
HERE=$(cd "$(dirname "$0")" && pwd)
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
EXPORT="${EXPORT_DIR:-$HERE}/links-turso-$STAMP.json"
SSH=(ssh -i "$SSH_KEY" "$SERVER")

echo "== 1/4 Export aus Turso (nur lesend) -> $EXPORT"
umask 077
node "$HERE/turso-export.mjs" >"$EXPORT"
ANZAHL=$(node -e "console.log(require('$EXPORT').links.length)")

echo "== 2/4 Hochladen"
REMOTE=/tmp/links-turso-$STAMP.json
scp -q -i "$SSH_KEY" "$EXPORT" "$SERVER:$REMOTE"

echo "== 3/4 Import als Benutzer shortener (idempotent)"
# Gleicher Benutzer und gleiche Datei wie der Dienst; WAL erlaubt den Import
# neben dem laufenden Dienst.
"${SSH[@]}" "sudo chown shortener: $REMOTE && \
  sudo -u shortener SHORTENER_DB=/var/lib/shortener/links.db /opt/shortener/shortener import $REMOTE"

echo "== 4/4 Prüfen: jeder Turso-Code zeigt auf dieselbe URL"
"${SSH[@]}" "sudo -u shortener SHORTENER_DB=/var/lib/shortener/links.db /opt/shortener/shortener verify $REMOTE; \
  rc=\$?; sudo rm -f $REMOTE; exit \$rc"

echo
echo "Abgleich fertig: $ANZAHL Links in Turso, alle auf dem Server vorhanden und identisch."
echo "Öffentliche Prüfung: ops/migration/verify-live.sh $EXPORT (Varianten im Kopf der Datei)"
