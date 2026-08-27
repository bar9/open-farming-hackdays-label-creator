//! URL-Shortener über einen einzigen Anbieter: da.gd.
//!
//! Geschichte dieser Datei in Kurzform: erst hing alles an tinyurl (in manchen
//! Netzen gesperrt), dann an einer Fallback-Kette aus is.gd/v.gd/spoo.me. Die
//! Kette hatte zwei praktische Probleme:
//!
//! 1. **Lokal unbrauchbar.** is.gd, v.gd, spoo.me, cleanuri und ulvis lehnen
//!    `http://localhost:8080/...` als ungültige URL ab. Beim Entwickeln fiel
//!    die Kette deshalb immer bis tinyurl durch — also genau zu dem Anbieter,
//!    den die Kette ersetzen sollte.
//! 2. **Gemeinsamer Ausfall.** is.gd und v.gd laufen auf derselben Software;
//!    fällt deren Datenbank aus, antworten beide mit
//!    "Error, database insert failed".
//!
//! da.gd löst beides: es kürzt auch `localhost`- und IP-Adressen (praktisch
//! für `dx serve` und Tests im LAN), ist kostenlos, werbefrei, braucht keinen
//! API-Key und sendet `Access-Control-Allow-Origin: *`. Die Antwort ist eine
//! Zeile Klartext mit dem Kurz-Link.

use std::fmt;

/// Der verwendete Anbieter. Bewusst ein Enum mit einer Variante: die
/// Antwort-Validierung und die Tests hängen daran, und ein zweiter Anbieter
/// liesse sich so ohne Umbau ergänzen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Provider {
    /// da.gd — kostenlos, werbefrei, ohne API-Key, `ACAO: *`, akzeptiert auch
    /// nicht-öffentliche Hosts.
    DaGd,
}

impl Provider {
    pub fn host(&self) -> &'static str {
        match self {
            Provider::DaGd => "da.gd",
        }
    }

    /// Anfrage-URL für die zu kürzende Adresse.
    pub fn request_url(&self, long_url: &str) -> String {
        match self {
            Provider::DaGd => format!(
                "https://da.gd/shorten?url={}",
                urlencoding::encode(long_url)
            ),
        }
    }

    /// Antwort des Anbieters in einen Kurz-Link übersetzen.
    ///
    /// Wichtig: Shortener antworten gern mit HTTP 200 und einer Fehlermeldung
    /// im Body ("Error, database insert failed"). Ohne Prüfung landete so ein
    /// Fehlertext im Eingabefeld — deshalb wird jede Antwort validiert.
    pub fn parse_response(&self, body: &str) -> Result<String, ShortenError> {
        let candidate = body.trim().to_string();
        if is_plausible_short_url(&candidate) {
            Ok(candidate)
        } else {
            Err(ShortenError::ProviderRejected(candidate))
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.host())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortenError {
    /// Anbieter hat geantwortet, aber keinen brauchbaren Link geliefert
    /// (Fehlermeldung, Rate-Limit, zu lange URL).
    ProviderRejected(String),
    /// Netzwerk-/CORS-Fehler: Anbieter gar nicht erreichbar.
    Unreachable(String),
}

impl fmt::Display for ShortenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShortenError::ProviderRejected(msg) => write!(f, "provider rejected: {}", msg),
            ShortenError::Unreachable(msg) => write!(f, "unreachable: {}", msg),
        }
    }
}

/// Ergebnis einer erfolgreichen Kürzung — inklusive Anbieter, damit die UI
/// transparent machen kann, wo das Rezept nun liegt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortLink {
    pub url: String,
    pub provider: Provider,
}

/// Grobe Plausibilitätsprüfung: eine kurze https-URL ohne Leerzeichen. Hält
/// Fehlertexte ("Error, database insert failed") aus dem Ergebnisfeld heraus.
fn is_plausible_short_url(candidate: &str) -> bool {
    candidate.starts_with("https://")
        && !candidate.contains(char::is_whitespace)
        && candidate.len() > "https://a.b/c".len()
        && candidate.len() < 200
}

/// Kürzt `long_url`. `fetch` kapselt den HTTP-Aufruf, damit die Logik ohne
/// Netz und ohne wasm testbar bleibt.
pub async fn shorten_with<F, Fut>(long_url: &str, fetch: F) -> Result<ShortLink, ShortenError>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<String, String>>,
{
    let provider = Provider::DaGd;
    match fetch(provider.request_url(long_url)).await {
        Ok(body) => match provider.parse_response(&body) {
            Ok(url) => Ok(ShortLink { url, provider }),
            Err(err) => {
                tracing::warn!("{} rejected the URL: {}", provider, err);
                Err(err)
            }
        },
        Err(err) => {
            tracing::warn!("{} unreachable: {}", provider, err);
            Err(ShortenError::Unreachable(err))
        }
    }
}

/// Produktions-Variante: kürzt über einen echten HTTP-Aufruf.
pub async fn shorten(long_url: &str) -> Result<ShortLink, ShortenError> {
    shorten_with(long_url, |url| async move {
        let response = gloo::net::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.ok() {
            return Err(format!("HTTP {}", response.status()));
        }
        response.text().await.map_err(|e| e.to_string())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_url_is_encoded() {
        let url = Provider::DaGd.request_url("https://declarino.ch/?a=1&b=2");
        assert!(url.starts_with("https://da.gd/shorten?url="));
        assert!(
            url.contains("%26b%3D2"),
            "query must be percent-encoded: {}",
            url
        );
    }

    #[test]
    fn parses_plaintext_short_link() {
        // da.gd antwortet mit einer Zeile Klartext (inkl. Zeilenumbruch).
        assert_eq!(
            Provider::DaGd.parse_response("https://da.gd/YraW\n").unwrap(),
            "https://da.gd/YraW"
        );
    }

    #[test]
    fn rejects_error_text_instead_of_putting_it_in_the_field() {
        // Fehlermeldungen kommen mit HTTP 200 im Body; ohne Prüfung landeten
        // sie früher als "Kurz-Link" im Eingabefeld.
        for body in [
            "Error: Invalid Url!",
            "Error, database insert failed",
            "",
            "http://da.gd/abc", // kein https → Mixed Content
        ] {
            assert!(
                Provider::DaGd.parse_response(body).is_err(),
                "must reject: {:?}",
                body
            );
        }
    }

    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        // Minimaler Executor: es gibt keine echten Wakeups, alle Test-Futures
        // sind sofort fertig.
        use std::task::{Context, Poll, Wake, Waker};
        struct Noop;
        impl Wake for Noop {
            fn wake(self: std::sync::Arc<Self>) {}
        }
        let waker = Waker::from(std::sync::Arc::new(Noop));
        let mut cx = Context::from_waker(&waker);
        let mut fut = Box::pin(fut);
        loop {
            if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
                return out;
            }
        }
    }

    #[test]
    fn shortens_via_da_gd() {
        let result = block_on(shorten_with("https://declarino.ch/?a=1", |url| async move {
            assert!(url.starts_with("https://da.gd/shorten?url="));
            Ok("https://da.gd/abc123\n".to_string())
        }))
        .unwrap();
        assert_eq!(result.provider, Provider::DaGd);
        assert_eq!(result.url, "https://da.gd/abc123");
    }

    #[test]
    fn shortens_localhost_urls_too() {
        // Der Grund für den Wechsel zu da.gd: beim Entwickeln mit `dx serve`
        // muss der Button funktionieren, statt still auf tinyurl auszuweichen.
        let result = block_on(shorten_with(
            "http://localhost:8080/open-farming-hackdays-label-creator/?a=1",
            |url| async move {
                assert!(url.contains("localhost"), "localhost must be forwarded");
                Ok("https://da.gd/YraW".to_string())
            },
        ))
        .unwrap();
        assert_eq!(result.url, "https://da.gd/YraW");
    }

    #[test]
    fn reports_network_failure() {
        let err = block_on(shorten_with("https://declarino.ch/?a=1", |_| async move {
            Err("NetworkError: Failed to fetch".to_string())
        }))
        .unwrap_err();
        assert_eq!(
            err,
            ShortenError::Unreachable("NetworkError: Failed to fetch".into())
        );
    }
}
