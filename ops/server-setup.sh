#!/usr/bin/env bash
# Einmalige (und wiederholbare) Einrichtung des declarino.ch-Servers.
#
# Idempotent: jeder Schritt prüft, ob er schon erledigt ist. Löscht nichts.
# Als Admin-Benutzer mit sudo ausführen, im Repo-Wurzelverzeichnis:
#
#   scp -r ops debian@<server>:/tmp/declarino-ops
#   ssh debian@<server> 'sudo bash /tmp/declarino-ops/server-setup.sh "<deploy-pubkey>"'
#
# Voraussetzungen (bereits auf dem Server erledigt): Debian 13, SSH nur mit
# Schlüssel, ufw mit 22/80/443, Caddy aus dem Debian-Paket.
#
# Danach genügt ein Lauf von deploy-production.yml: er liefert die Seite und
# das Programm /opt/shortener/shortener; der Dienst legt seine Datenbank beim
# ersten Start selbst an.
set -euo pipefail

DEPLOY_PUBKEY=${1:?Aufruf: $0 "<öffentlicher Schlüssel für deploy>"}
OPS=$(cd "$(dirname "$0")" && pwd)

[ "$(id -u)" = 0 ] || { echo "Bitte mit sudo ausführen." >&2; exit 1; }

echo "== Pakete"
# sqlite3 für die nächtliche Sicherung (.backup) und für Prüfungen von Hand.
apt-get install -y --no-install-recommends sqlite3 rsync >/dev/null

echo "== Benutzer deploy (CI) und shortener (Dienst)"
id deploy >/dev/null 2>&1 || useradd --create-home --shell /bin/bash deploy
id shortener >/dev/null 2>&1 ||
  useradd --system --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin shortener

# `restrict` schaltet Weiterleitungen, Agent und PTY ab; CI braucht nur
# Befehle und rsync.
install -d -m 700 -o deploy -g deploy /home/deploy/.ssh
echo "restrict ${DEPLOY_PUBKEY}" >/home/deploy/.ssh/authorized_keys
chown deploy:deploy /home/deploy/.ssh/authorized_keys
chmod 600 /home/deploy/.ssh/authorized_keys

echo "== Verzeichnisse"
install -d -m 755 -o deploy -g deploy /var/www/declarino /var/www/declarino/releases
install -d -m 755 -o deploy -g deploy /opt/shortener
install -d -m 700 -o shortener -g shortener /var/backups/shortener

echo "== sudoers: deploy darf genau einen Befehl"
TMP=$(mktemp)
echo 'deploy ALL=(root) NOPASSWD: /usr/bin/systemctl restart shortener' >"$TMP"
chmod 440 "$TMP"
visudo -cf "$TMP"
mv "$TMP" /etc/sudoers.d/deploy-shortener
visudo -c >/dev/null

echo "== systemd-Units"
install -m 644 "$OPS/shortener/shortener.service" /etc/systemd/system/shortener.service
install -m 644 "$OPS/shortener/shortener-backup.service" /etc/systemd/system/shortener-backup.service
install -m 644 "$OPS/shortener/shortener-backup.timer" /etc/systemd/system/shortener-backup.timer
systemctl daemon-reload
systemctl enable shortener.service >/dev/null
systemctl enable --now shortener-backup.timer >/dev/null
# Startet erst, wenn ein Deploy /opt/shortener/shortener geliefert hat
# (ConditionPathExists in der Unit).
systemctl start shortener.service || true

echo "== Caddy"
if ! cmp -s "$OPS/caddy/Caddyfile" /etc/caddy/Caddyfile; then
  cp -a /etc/caddy/Caddyfile "/etc/caddy/Caddyfile.vorher-$(date +%Y%m%d%H%M%S)"
  install -m 644 "$OPS/caddy/Caddyfile" /etc/caddy/Caddyfile
fi
caddy validate --config /etc/caddy/Caddyfile >/dev/null
systemctl reload caddy

echo "Fertig."
