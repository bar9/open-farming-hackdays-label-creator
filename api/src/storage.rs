//! Speicher: eine lokale SQLite-Datei auf dem Server.
//!
//! Kurz-Links sind dauerhafter Zustand: Einer auf einem gedruckten Etikett
//! lässt sich nicht mehr ändern. Der Bestand ist winzig (~1 KB je Eintrag),
//! eine einzelne Datei mit nächtlicher Sicherung (siehe ops/shortener/) reicht.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// Schema, unverändert aus der früheren Turso-Datenbank übernommen.
///
/// Wird bei jedem Öffnen angelegt, falls es fehlt: ein frischer Server ist so
/// ohne manuellen Migrationsschritt sofort benutzbar.
const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS links (
  code TEXT PRIMARY KEY,
  url TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
)";

/// Ein gespeicherter Kurz-Link, auch das Format von Export und Import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub code: String,
    pub url: String,
    /// Unix-Sekunden. `None` nur in Importdateien ohne Anlagedatum.
    pub created_at: Option<i64>,
}

/// Ergebnis eines einzelnen Imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportOutcome {
    /// Neu angelegt.
    Written,
    /// Code mit derselben URL schon vorhanden: nichts zu tun.
    Existing,
    /// Code ist mit einer *anderen* URL belegt. Wird nie überschrieben, sonst
    /// landete ein bereits gedruckter Link stillschweigend woanders.
    Conflict,
}

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Öffnet (oder erzeugt) die Datenbankdatei und legt das Schema an.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        // WAL: Leser blockieren den Schreiber nicht, und die nächtliche
        // Sicherung (sqlite3 .backup) läuft neben dem Dienst.
        let mode: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "journal_mode=WAL nicht aktiv (ist: {mode})"
            )));
        }
        Self::init(conn)
    }

    /// Flüchtige Datenbank für Tests.
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Self> {
        // Import-Lauf und Dienst können gleichzeitig schreiben; kurz warten
        // statt sofort mit SQLITE_BUSY abzubrechen.
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        // Ein Panic in einem anderen Thread macht die Verbindung nicht
        // unbrauchbar; weiterarbeiten statt den Dienst lahmzulegen.
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Eintrag anlegen, wenn der Code noch frei ist.
    /// `true` = geschrieben, `false` = Code bereits belegt.
    /// Bewusst ohne Ablaufdatum: geteilte und gedruckte Links dürfen nicht
    /// verschwinden.
    pub fn store_if_absent(&self, code: &str, url: &str) -> rusqlite::Result<bool> {
        // ON CONFLICT DO NOTHING: nur schreiben, wenn der Code frei ist. Die
        // Zahl geänderter Zeilen unterscheidet "neu" von "schon belegt", ohne
        // Lücke zwischen Prüfen und Schreiben.
        let changed = self.conn().execute(
            "INSERT INTO links (code, url) VALUES (?1, ?2) ON CONFLICT(code) DO NOTHING",
            params![code, url],
        )?;
        Ok(changed > 0)
    }

    /// Ziel-URL zu einem Code, oder `None`.
    pub fn lookup(&self, code: &str) -> rusqlite::Result<Option<String>> {
        self.conn()
            .query_row("SELECT url FROM links WHERE code = ?1", [code], |r| {
                r.get(0)
            })
            .optional()
    }

    /// Alle Einträge, für Sicherung und Prüfung.
    pub fn list_all(&self) -> rusqlite::Result<Vec<Link>> {
        let conn = self.conn();
        let mut stmt =
            conn.prepare("SELECT code, url, created_at FROM links ORDER BY created_at, code")?;
        let rows = stmt.query_map([], |r| {
            Ok(Link {
                code: r.get(0)?,
                url: r.get(1)?,
                created_at: r.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn count(&self) -> rusqlite::Result<i64> {
        self.conn()
            .query_row("SELECT COUNT(*) FROM links", [], |r| r.get(0))
    }

    /// Mehrere Einträge in einer Transaktion importieren, mit Anlagedatum.
    ///
    /// Idempotent: ein zweiter Lauf mit derselben (oder einer neueren) Datei
    /// legt nur an, was fehlt, und ändert nichts Bestehendes.
    pub fn import(&self, links: &[Link]) -> rusqlite::Result<Vec<ImportOutcome>> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let mut outcomes = Vec::with_capacity(links.len());
        {
            let mut insert = tx.prepare(
                "INSERT INTO links (code, url, created_at)
                 VALUES (?1, ?2, COALESCE(?3, unixepoch()))
                 ON CONFLICT(code) DO NOTHING",
            )?;
            let mut select = tx.prepare("SELECT url FROM links WHERE code = ?1")?;
            for link in links {
                let outcome = if insert.execute(params![link.code, link.url, link.created_at])? > 0
                {
                    ImportOutcome::Written
                } else {
                    let existing: String = select.query_row([&link.code], |r| r.get(0))?;
                    if existing == link.url {
                        ImportOutcome::Existing
                    } else {
                        ImportOutcome::Conflict
                    }
                };
                outcomes.push(outcome);
            }
        }
        tx.commit()?;
        Ok(outcomes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_lookup_list() {
        let store = Store::open_in_memory().unwrap();
        assert!(
            store
                .store_if_absent("abc1234", "https://www.declarino.ch/a")
                .unwrap()
        );
        assert!(
            !store
                .store_if_absent("abc1234", "https://www.declarino.ch/b")
                .unwrap()
        );
        assert_eq!(
            store.lookup("abc1234").unwrap().as_deref(),
            Some("https://www.declarino.ch/a")
        );
        assert_eq!(store.lookup("fehlt00").unwrap(), None);
        let all = store.list_all().unwrap();
        assert_eq!(all.len(), 1);
        assert!(all[0].created_at.unwrap() > 1_700_000_000);
    }

    #[test]
    fn import_is_idempotent_and_never_overwrites() {
        let store = Store::open_in_memory().unwrap();
        let links = vec![
            Link {
                code: "aaaaaaa".into(),
                url: "https://www.declarino.ch/1".into(),
                created_at: Some(1_787_836_870),
            },
            Link {
                code: "bbbbbbb".into(),
                url: "https://www.declarino.ch/2".into(),
                created_at: None,
            },
        ];
        use ImportOutcome::*;
        assert_eq!(store.import(&links).unwrap(), vec![Written, Written]);
        assert_eq!(store.import(&links).unwrap(), vec![Existing, Existing]);
        let all = store.list_all().unwrap();
        assert_eq!(
            all[0].created_at,
            Some(1_787_836_870),
            "Anlagedatum bleibt erhalten"
        );

        let clash = vec![Link {
            code: "aaaaaaa".into(),
            url: "https://www.declarino.ch/anders".into(),
            created_at: None,
        }];
        assert_eq!(store.import(&clash).unwrap(), vec![Conflict]);
        assert_eq!(
            store.lookup("aaaaaaa").unwrap().as_deref(),
            Some("https://www.declarino.ch/1")
        );
    }

    #[test]
    fn file_database_uses_wal_and_creates_schema() {
        let dir = std::env::temp_dir().join(format!("shortener-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("links.db");
        {
            let store = Store::open(&path).unwrap();
            store
                .store_if_absent("ccccccc", "https://declarino.ch/")
                .unwrap();
        }
        // Wiederöffnen: Schema-Anlage ist idempotent, Daten bleiben.
        let store = Store::open(&path).unwrap();
        assert_eq!(store.count().unwrap(), 1);
        let mode: String = store
            .conn()
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode, "wal");
        drop(store);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
