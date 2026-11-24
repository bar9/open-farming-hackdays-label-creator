# Betrieb: Server für www.declarino.ch

Produktion läuft auf einem Infomaniak-VPS (Debian 13, 1 CPU, 2 GB) mit Caddy.
Staging bleibt auf GitHub Pages (`deploy.yml`).

```
Internet ──► Caddy :443 ─┬─ /api/*, /s/*  ──► shortener 127.0.0.1:3000 ──► /var/lib/shortener/links.db
                         └─ alles andere  ──► /var/www/declarino/current (SPA, try_files)
             Caddy :443  declarino.ch ──► 301 https://www.declarino.ch{uri}
             Caddy 127.0.0.1:8081 ──► wie www, nur lokal (Smoke-Test des Deploys)
```

| Datei | Ziel auf dem Server |
|---|---|
| `caddy/Caddyfile` | `/etc/caddy/Caddyfile` |
| `shortener/shortener.service` | `/etc/systemd/system/` |
| `shortener/shortener-backup.{service,timer}` | `/etc/systemd/system/` |
| `activate-release.sh` | `/opt/shortener/` (liefert der Deploy) |
| `server-setup.sh` | Einrichtung, einmalig bzw. bei Änderungen |
| `caddy/test-routing.sh` | Routing-Test in CI (kein Server nötig) |
| `migration/` | **vorübergehend**: Abgleich mit der alten Turso-Datenbank |

## Benutzer und Rechte

* **`deploy`**: Login für CI, nur per Schlüssel (`restrict` in
  `authorized_keys`). Besitzt `/var/www/declarino` und `/opt/shortener`. Darf
  per sudo genau `/usr/bin/systemctl restart shortener` (sonst nichts,
  `/etc/sudoers.d/deploy-shortener`).
* **`shortener`**: Systembenutzer ohne Login, unter dem der Dienst läuft.
  Einziger schreibender Zugriff: `/var/lib/shortener` (Datenbank) und für die
  Sicherung `/var/backups/shortener`. Der Dienst ist per systemd gehärtet
  (`ProtectSystem=strict`, `ProtectHome`, `PrivateTmp`, `NoNewPrivileges`,
  `MemoryMax=128M`, …) und hört nur auf `127.0.0.1:3000`.

## Grundeinrichtung des Systems

Von Hand erledigt am 16.11.2026, vor `server-setup.sh` (Stand geprüft am
24.11.2026). Das Setup-Skript setzt diese Schritte voraus und macht sie nicht
selbst.

* **Ausgangslage:** Infomaniak VPS Lite mit dem Debian-13-Image aus dem
  Infomaniak-Manager. Admin-Benutzer `debian` (sudo ohne Passwort), Login mit
  dem beim Bestellen erzeugten Schlüssel:
  `ssh -i <schlüssel> debian@83.228.242.209`. Zeitzone UTC, Uhr per systemd
  synchronisiert.
* **SSH gehärtet:** `/etc/ssh/sshd_config.d/10-hardening.conf`:

  ```
  PasswordAuthentication no
  PermitRootLogin no
  KbdInteractiveAuthentication no
  ```

  Danach `sudo systemctl reload ssh`, **vorher** in einer zweiten Sitzung
  prüfen, dass der Schlüssel-Login klappt.
* **Firewall:** `ufw` installiert, eingehend alles gesperrt ausser

  ```bash
  sudo ufw allow OpenSSH
  sudo ufw allow 80,443/tcp
  sudo ufw enable
  ```

  Zusätzlich hat der Infomaniak-Manager eine eigene Firewall vor dem VPS. Dort
  mussten 80 und 443 eigens geöffnet werden, ohne das kommen Web-Anfragen nie
  an, obwohl ufw sie erlaubt.
* **Caddy:** `sudo apt install caddy` (Debian-Paket, 2.6.2). Läuft als
  systemd-Dienst `caddy` und holt die Zertifikate selbst.
* **Pakete von `server-setup.sh`:** `sqlite3` (Sicherung) und `rsync`
  (Deploy).
* **Automatische Updates:** `unattended-upgrades` ist im Debian-Image schon
  aktiv (`/etc/apt/apt.conf.d/20auto-upgrades`: Paketlisten und Upgrade
  täglich). Eingespielt werden die Pakete aus Debian und Debian-Security,
  also auch Caddy. Die Timer `apt-daily.timer` und `apt-daily-upgrade.timer`
  steuern das, das Protokoll liegt unter
  `/var/log/unattended-upgrades/unattended-upgrades.log`.

  **Kein automatischer Neustart:** Ein neuer Kernel wird installiert, wirkt
  aber erst nach einem Reboot. Ob einer fällig ist, zeigt der Vergleich von
  `uname -r` mit `ls /boot/vmlinuz-*`. Nach einem Neustart (`sudo reboot`)
  starten Caddy, der Shortener und der Sicherungs-Timer von selbst wieder
  (alle `enabled`). Danach prüfen: `curl -I https://www.declarino.ch` und ein Kurz-Link.

  ```bash
  systemctl list-timers 'apt-daily*'
  sudo tail /var/log/unattended-upgrades/unattended-upgrades.log
  ```

## Neuen Server einrichten

1. Voraussetzungen: die Grundeinrichtung oben (SSH nur mit Schlüssel, ufw mit
   22/80/443, Caddy aus dem Debian-Paket, Firewall im Infomaniak-Manager).
2. Deploy-Schlüssel lokal erzeugen und den Server einrichten:

   ```bash
   ssh-keygen -t ed25519 -N '' -C github-actions-declarino -f ~/.ssh/declarino_deploy
   scp -r ops debian@<server>:/tmp/declarino-ops
   ssh debian@<server> "sudo bash /tmp/declarino-ops/server-setup.sh '$(cat ~/.ssh/declarino_deploy.pub)'"
   ```

   Das Skript ist idempotent und löscht nichts. Eine geänderte Caddyfile wird
   vorher gesichert (`/etc/caddy/Caddyfile.vorher-*`) und mit `caddy validate`
   geprüft, bevor Caddy neu lädt.
3. GitHub-Secrets setzen:

   ```bash
   gh secret set VPS_SSH_KEY < ~/.ssh/declarino_deploy
   ssh-keyscan -t ed25519,ecdsa,rsa <server> | gh secret set VPS_KNOWN_HOSTS
   printf '<server>' | gh secret set VPS_HOST
   ```

   Den Fingerabdruck aus `ssh-keyscan` mit `ssh-keygen -lf
   /etc/ssh/ssh_host_*_key.pub` auf dem Server vergleichen.
4. `deploy-production.yml` von Hand starten (`gh workflow run
   deploy-production.yml`). Der Dienst legt seine Datenbank beim ersten Start
   selbst an.

## Deploy

`deploy-production.yml` läuft nach jedem erfolgreichen "Check & Test" auf
`main` und von Hand (`workflow_dispatch`):

1. baut die App (ohne Feature-Flags) und den Dienst (statisch, musl),
2. lädt die Seite per rsync in ein neues `releases/<UTC-Zeit>-<sha>/` und das
   Programm als `/opt/shortener/shortener.new`,
3. `activate-release.sh`: ersetzt das Programm, schaltet `current` atomar um
   (`ln -sfn` auf einen Hilfsnamen, dann `mv -Tf`), behält die letzten 5
   Releases und startet den Dienst neu,
4. Smoke-Test auf dem Server über `127.0.0.1:8081` (gleiche Caddy-Regeln wie
   www, unabhängig von DNS und Zertifikat).

**Zurückrollen** auf ein früheres Release (als `deploy`):

```bash
cd /var/www/declarino && ls releases/
ln -sfn releases/<älteres-release> current.tmp && mv -Tf current.tmp current
```

## Sicherung

Der Timer `shortener-backup.timer` sichert jede Nacht (03:15 UTC, ±15 min) mit
`sqlite3 .backup` nach `/var/backups/shortener/links-<zeit>.db`, prüft die
Kopie mit `PRAGMA integrity_check` und behält die letzten 14.

```bash
systemctl list-timers shortener-backup.timer
sudo systemctl start shortener-backup.service    # sofort sichern
journalctl -u shortener-backup.service
```

Die Kopien liegen auf demselben Server. Für Schutz gegen den Verlust des
ganzen Servers zusätzlich ab und zu herunterladen oder das Infomaniak-Backup
des VPS aktivieren.

## Umstellung von der alten Plattform (vorübergehend)

`migration/sync-links.sh` liest alle Links aus der alten Turso-Datenbank (nur
lesend), importiert sie auf dem Server (idempotent) und prüft, dass jeder Code
auf dieselbe URL zeigt. Beliebig oft ausführbar, zuletzt direkt vor der
DNS-Umstellung:

```bash
TURSO_DATABASE_URL=… TURSO_AUTH_TOKEN=… ops/migration/sync-links.sh
```

Nach der Umstellung und dem letzten Abgleich wird `ops/migration/` gelöscht
(`verify-live.sh` vorher nach `ops/` verschieben, falls es weiter gebraucht
wird).
