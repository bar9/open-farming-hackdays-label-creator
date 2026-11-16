#!/usr/bin/env bash
# Prüft jeden Code einer Sicherungsdatei gegen eine laufende Instanz: HTTP 301
# und Location exakt gleich der hinterlegten URL. Belegt damit, was jemand mit
# einem gedruckten Etikett tatsächlich erlebt.
#
#   ops/migration/verify-live.sh links.json
#       # öffentlich über DNS: https://www.declarino.ch
#   RESOLVE=83.228.242.209 ops/migration/verify-live.sh links.json
#       # über Caddy auf dem neuen Server, unabhängig vom DNS (braucht dessen
#       # Zertifikat, also erst nach der Umstellung)
#   ssh -i ~/.ssh/infomaniak -fNL 13000:127.0.0.1:3000 debian@83.228.242.209
#   BASE=http://127.0.0.1:13000 ops/migration/verify-live.sh links.json
#       # direkt gegen den Dienst per SSH-Tunnel, auch vor der Umstellung
#
# Braucht: node, curl.
set -uo pipefail

DATEI=${1:?Aufruf: $0 <links.json>}
BASE=${BASE:-https://www.declarino.ch}
CURL=(curl -sS -o /dev/null --max-time 10 -w '%{http_code} %{redirect_url}')
[ -n "${RESOLVE:-}" ] && CURL+=(--resolve "www.declarino.ch:443:$RESOLVE")

ok=0
bad=0
while IFS=$'\t' read -r code url; do
  read -r status location < <("${CURL[@]}" "$BASE/s/$code")
  if [ "$status" = 301 ] && [ "$location" = "$url" ]; then
    ok=$((ok + 1))
  else
    bad=$((bad + 1))
    echo "  FEHLT: /s/$code -> HTTP $status" >&2
  fi
done < <(node -e "for (const l of require(require('path').resolve('$DATEI')).links) console.log(l.code + '\t' + l.url)")

echo "$ok Links funktionieren, $bad fehlerhaft ($BASE)."
[ "$bad" = 0 ]
