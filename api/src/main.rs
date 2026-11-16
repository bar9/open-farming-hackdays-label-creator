//! Einstiegspunkt: Dienst starten oder den Bestand pflegen.
//!
//! ```text
//! shortener                       Dienst starten (wie `serve`)
//! shortener serve
//! shortener import <links.json>   Links übernehmen (idempotent)
//! shortener export                alle Links als JSON nach stdout
//! shortener verify <links.json>   prüft, dass jeder Code der Datei in der
//!                                 Datenbank auf dieselbe URL zeigt
//! ```
//!
//! Umgebung:
//! * `SHORTENER_DB`: Datenbankdatei (Standard: `$STATE_DIRECTORY/links.db`,
//!   sonst `/var/lib/shortener/links.db`)
//! * `SHORTENER_LISTEN`: Adresse (Standard: `127.0.0.1:3000`). Nur lokal,
//!   nach aussen spricht Caddy.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use serde::Deserialize;
use shortener::rules::is_allowed_target;
use shortener::storage::{ImportOutcome, Link, Store};

const USAGE: &str = "Kurz-Link-Dienst für declarino.ch

  shortener [serve]
  shortener import <links.json>
  shortener export > links.json
  shortener verify <links.json>
  shortener version

Umgebung: SHORTENER_DB (Datenbankdatei), SHORTENER_LISTEN (Standard 127.0.0.1:3000)";

fn db_path() -> PathBuf {
    if let Some(path) = std::env::var_os("SHORTENER_DB") {
        return path.into();
    }
    // Von systemd gesetzt (StateDirectory=shortener).
    if let Some(dir) = std::env::var_os("STATE_DIRECTORY") {
        return PathBuf::from(dir).join("links.db");
    }
    PathBuf::from("/var/lib/shortener/links.db")
}

fn open_store() -> Result<Store, String> {
    let path = db_path();
    Store::open(&path).map_err(|e| format!("Datenbank {} nicht nutzbar: {e}", path.display()))
}

/// Format der Sicherungsdateien. `count` und weitere Felder sind optional,
/// damit auch der Export des früheren Turso-Dienstes lesbar bleibt.
#[derive(Deserialize)]
struct LinkFile {
    links: Vec<Link>,
}

fn read_links(path: &str) -> Result<Vec<Link>, String> {
    let raw = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let file: LinkFile = serde_json::from_slice(&raw).map_err(|e| format!("{path}: {e}"))?;
    Ok(file.links)
}

fn run_import(path: &str) -> Result<ExitCode, String> {
    let links = read_links(path)?;
    // Auch beim Import gilt die Allowlist: eine manipulierte Sicherungsdatei
    // darf keine fremden Ziele einschleusen.
    let (accepted, rejected): (Vec<Link>, Vec<Link>) =
        links.into_iter().partition(|l| is_allowed_target(&l.url));
    for link in &rejected {
        eprintln!("  übersprungen (fremdes Ziel): {}", link.code);
    }
    let store = open_store()?;
    let outcomes = store.import(&accepted).map_err(|e| e.to_string())?;
    let mut written = 0;
    let mut existing = 0;
    let mut conflicts = 0;
    for (link, outcome) in accepted.iter().zip(&outcomes) {
        match outcome {
            ImportOutcome::Written => written += 1,
            ImportOutcome::Existing => existing += 1,
            ImportOutcome::Conflict => {
                conflicts += 1;
                eprintln!("  KONFLIKT: {} ist mit einer anderen URL belegt", link.code);
            }
        }
    }
    let total = store.count().map_err(|e| e.to_string())?;
    eprintln!(
        "Import: {written} neu, {existing} bereits vorhanden, {conflicts} Konflikte, {} abgelehnt. Bestand: {total}.",
        rejected.len()
    );
    Ok(if conflicts == 0 && rejected.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn run_export() -> Result<ExitCode, String> {
    let links = open_store()?.list_all().map_err(|e| e.to_string())?;
    let exported_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    eprintln!("{} Einträge gelesen.", links.len());
    let out = serde_json::json!({
        "exported_at_unix": exported_at,
        "backend": "sqlite",
        "count": links.len(),
        "links": links,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?
    );
    Ok(ExitCode::SUCCESS)
}

/// Prüft die Datenbank gegen eine Sicherungsdatei: jeder Code muss auf genau
/// dieselbe URL zeigen. Die öffentliche Seite prüft ops/shortener/verify-live.sh.
fn run_verify(path: &str) -> Result<ExitCode, String> {
    let links = read_links(path)?;
    let store = open_store()?;
    let mut ok = 0;
    let mut bad = 0;
    for link in &links {
        match store.lookup(&link.code).map_err(|e| e.to_string())? {
            Some(url) if url == link.url => ok += 1,
            Some(_) => {
                bad += 1;
                eprintln!("  ABWEICHEND: {}", link.code);
            }
            None => {
                bad += 1;
                eprintln!("  FEHLT: {}", link.code);
            }
        }
    }
    let total = store.count().map_err(|e| e.to_string())?;
    eprintln!(
        "{ok} von {} Links stimmen, {bad} fehlerhaft. Bestand in der Datenbank: {total}.",
        links.len()
    );
    Ok(if bad == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn run_serve() -> Result<ExitCode, String> {
    let store = Arc::new(open_store()?);
    let listen = std::env::var("SHORTENER_LISTEN").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(&listen)
            .await
            .map_err(|e| format!("{listen}: {e}"))?;
        eprintln!(
            "shortener hört auf {listen}, Datenbank {}",
            db_path().display()
        );
        axum::serve(listener, shortener::router(store))
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| e.to_string())
    })?;
    Ok(ExitCode::SUCCESS)
}

/// Bei SIGTERM (systemctl restart/stop) laufende Anfragen beenden und die
/// Datenbank sauber schliessen.
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let mut term = signal(SignalKind::terminate()).expect("SIGTERM-Handler");
    tokio::select! {
        _ = term.recv() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["serve"] => run_serve(),
        ["import", file] => run_import(file),
        ["export"] => run_export(),
        ["verify", file] => run_verify(file),
        ["version"] => {
            println!("shortener {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("Fehler: {message}");
            ExitCode::FAILURE
        }
    }
}
