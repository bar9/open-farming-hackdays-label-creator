#!/usr/bin/env bash
# Läuft auf dem Server als Benutzer deploy, aufgerufen von
# deploy-production.yml, nachdem die neue Version hochgeladen ist:
#
#   /var/www/declarino/releases/<release>/   die neue Seite (vollständig)
#   /opt/shortener/shortener.new             das neue Programm
#
# Schaltet beides scharf, räumt alte Releases weg und startet den Dienst neu.
#
# Aufruf: activate-release.sh <release-name>
set -euo pipefail

RELEASE=${1:?Aufruf: $0 <release-name>}
case "$RELEASE" in
  *[!0-9A-Za-z._-]*) echo "Ungültiger Release-Name: $RELEASE" >&2; exit 2 ;;
esac

BASE=/var/www/declarino
DIR="$BASE/releases/$RELEASE"
KEEP=5

[ -f "$DIR/index.html" ] || { echo "$DIR/index.html fehlt" >&2; exit 1; }

# Programm austauschen: erst prüfen, dass das neue läuft, dann per rename
# ersetzen (atomar; der laufende Prozess behält seine alte Datei).
if [ -f /opt/shortener/shortener.new ]; then
  chmod 755 /opt/shortener/shortener.new
  /opt/shortener/shortener.new version ||
    { echo "neues shortener-Programm startet nicht" >&2; exit 1; }
  mv -f /opt/shortener/shortener.new /opt/shortener/shortener
fi

# Seite umschalten: Symlink unter Hilfsnamen anlegen und per rename über
# `current` legen, damit nie ein Moment ohne gültiges `current` entsteht.
ln -sfn "releases/$RELEASE" "$BASE/current.tmp"
mv -Tf "$BASE/current.tmp" "$BASE/current"
echo "current -> releases/$RELEASE"

# Die letzten $KEEP Releases behalten, das aktive in jedem Fall. Die Namen
# beginnen mit einem UTC-Zeitstempel (YYYYmmddTHHMMSSZ-<sha>), sortieren also
# chronologisch.
cd "$BASE/releases"
ls -1 | sort -r | tail -n +$((KEEP + 1)) | while read -r alt; do
  [ "$alt" = "$RELEASE" ] && continue
  rm -rf -- "$alt"
  echo "entfernt: $alt"
done

sudo /usr/bin/systemctl restart shortener
for _ in $(seq 25); do
  systemctl is-active --quiet shortener && break
  sleep 0.2
done
systemctl is-active --quiet shortener || {
  echo "shortener läuft nicht" >&2
  exit 1
}
echo "shortener neu gestartet."
