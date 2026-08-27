//! URL-Shortener: bevorzugt interstitial-freie Anbieter, mit da.gd als
//! Rückfallebene und als einzigem Weg für lokale Adressen.
//!
//! Zwei Anforderungen stehen hier im Konflikt und erklären den Aufbau:
//!
//! 1. **Produktion soll direkt weiterleiten.** da.gd blendet für jeden Link,
//!    der jünger als eine Stunde ist, eine Zwischenseite ein
//!    (`shorten.shorturl_interstitial_cooldown => 3600` in dagd/dagd). Genau
//!    frische Links werden aber geteilt — der Empfänger sähe fast immer erst
//!    die Warnseite. is.gd/v.gd leiten dagegen sofort weiter.
//! 2. **Lokal soll der Button funktionieren.** is.gd, v.gd, spoo.me, cleanuri
//!    und ulvis lehnen `http://localhost:8080/...` als ungültig ab. Nur da.gd
//!    kürzt nicht-öffentliche Hosts — und dort ist die Zwischenseite egal,
//!    weil der Link ohnehin nur auf diesem Rechner funktioniert.
//!
//! Deshalb: für öffentliche Adressen zuerst is.gd, dann v.gd, dann da.gd; für
//! lokale Adressen direkt da.gd. Alle Endpunkte sind kostenlos, werbefrei,
//! ohne API-Key und senden `Access-Control-Allow-Origin`.

use std::fmt;

/// Ein Anbieter der Fallback-Kette.
// Die Varianten enden alle auf "Gd" — das sind aber die echten Domainnamen
// der Dienste, ein Umbenennen würde sie nur verschleiern.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Provider {
    /// is.gd — `ACAO: *`, JSON-Antwort, leitet ohne Zwischenseite weiter.
    IsGd,
    /// v.gd — gleiche Software/API wie is.gd, aber eigene Domain: fällt nicht
    /// unter dieselben Sperrlisten.
    VGd,
    /// da.gd — kostenlos, werbefrei, akzeptiert als einziger auch localhost/IPs.
    /// Zeigt bei Links unter einer Stunde eine Zwischenseite, steht deshalb
    /// für öffentliche Adressen hinten.
    DaGd,
}

impl Provider {
    /// Anbieter für `long_url`, in Reihenfolge der Versuche.
    ///
    /// is.gd und v.gd laufen auf derselben Software: fällt deren Datenbank aus
    /// ("Error, database insert failed"), scheitern beide gleichzeitig —
    /// da.gd als unabhängige Infrastruktur fängt das ab.
    pub fn chain_for(long_url: &str) -> &'static [Provider] {
        if is_publicly_reachable(long_url) {
            &[Provider::IsGd, Provider::VGd, Provider::DaGd]
        } else {
            // Nur da.gd kürzt lokale Adressen; die anderen würden mit
            // "invalid URL" antworten.
            &[Provider::DaGd]
        }
    }

    pub fn host(&self) -> &'static str {
        match self {
            Provider::IsGd => "is.gd",
            Provider::VGd => "v.gd",
            Provider::DaGd => "da.gd",
        }
    }

    /// Anfrage-URL für die zu kürzende Adresse.
    pub fn request_url(&self, long_url: &str) -> String {
        let encoded = urlencoding::encode(long_url);
        match self {
            Provider::IsGd => format!("https://is.gd/create.php?format=json&url={}", encoded),
            Provider::VGd => format!("https://v.gd/create.php?format=json&url={}", encoded),
            Provider::DaGd => format!("https://da.gd/shorten?url={}", encoded),
        }
    }

    /// Antwort des Anbieters in einen Kurz-Link übersetzen.
    ///
    /// Wichtig: diese Dienste antworten mit HTTP 200 und einer Fehlermeldung
    /// im Body ("Error, database insert failed", `{"errorcode":1,...}`). Ohne
    /// Prüfung landete so ein Fehlertext im Eingabefeld — deshalb wird jede
    /// Antwort validiert.
    pub fn parse_response(&self, body: &str) -> Result<String, ShortenError> {
        let body = body.trim();
        let candidate = match self {
            Provider::IsGd | Provider::VGd => extract_json_shorturl(body)?,
            Provider::DaGd => body.to_string(),
        };
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
    /// Alle Anbieter der Kette sind gescheitert.
    AllProvidersFailed,
}

impl fmt::Display for ShortenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShortenError::ProviderRejected(msg) => write!(f, "provider rejected: {}", msg),
            ShortenError::Unreachable(msg) => write!(f, "unreachable: {}", msg),
            ShortenError::AllProvidersFailed => f.write_str("all providers failed"),
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

fn extract_json_shorturl(body: &str) -> Result<String, ShortenError> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| ShortenError::ProviderRejected(body.to_string()))?;
    if let Some(short) = value.get("shorturl").and_then(|v| v.as_str()) {
        return Ok(short.to_string());
    }
    let message = value
        .get("errormessage")
        .and_then(|v| v.as_str())
        .unwrap_or(body);
    Err(ShortenError::ProviderRejected(message.to_string()))
}

/// Grobe Plausibilitätsprüfung: eine kurze https-URL ohne Leerzeichen. Hält
/// Fehlertexte ("Error, database insert failed") aus dem Ergebnisfeld heraus.
fn is_plausible_short_url(candidate: &str) -> bool {
    candidate.starts_with("https://")
        && !candidate.contains(char::is_whitespace)
        && candidate.len() > "https://a.b/c".len()
        && candidate.len() < 200
}

/// Host einer URL grob extrahieren (ohne url-Crate, reicht für die Prüfung).
fn host_of(url: &str) -> String {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let host = host.split_once(':').map(|(h, _)| h).unwrap_or(host);
    host.trim_matches(['[', ']']).to_lowercase()
}

/// Ob die Adresse ausserhalb des eigenen Rechners/Netzes erreichbar ist.
/// Entscheidet, welche Anbieter überhaupt in Frage kommen.
pub fn is_publicly_reachable(url: &str) -> bool {
    let host = host_of(url);
    if host.is_empty() {
        return false;
    }
    if host == "localhost"
        || host == "::1"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".test")
    {
        return false;
    }
    // Private IPv4-Bereiche (RFC 1918) und Loopback.
    let octets: Vec<&str> = host.split('.').collect();
    if octets.len() == 4 && octets.iter().all(|o| o.parse::<u8>().is_ok()) {
        let n: Vec<u8> = octets.iter().map(|o| o.parse::<u8>().unwrap()).collect();
        return !(n[0] == 127
            || n[0] == 10
            || (n[0] == 192 && n[1] == 168)
            || (n[0] == 172 && (16..=31).contains(&n[1])));
    }
    // Ein Hostname ohne Punkt ist ein reiner Intranet-Name.
    host.contains('.')
}

/// Kürzt `long_url`, indem die passenden Anbieter der Reihe nach probiert
/// werden. `fetch` kapselt den HTTP-Aufruf, damit die Kettenlogik ohne Netz
/// und ohne wasm testbar bleibt.
pub async fn shorten_with<F, Fut>(long_url: &str, fetch: F) -> Result<ShortLink, ShortenError>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<String, String>>,
{
    let mut last_error = ShortenError::AllProvidersFailed;
    for provider in Provider::chain_for(long_url) {
        let provider = *provider;
        match fetch(provider.request_url(long_url)).await {
            Ok(body) => match provider.parse_response(&body) {
                Ok(url) => return Ok(ShortLink { url, provider }),
                Err(err) => {
                    tracing::warn!("{} rejected the URL: {}", provider, err);
                    last_error = err;
                }
            },
            Err(err) => {
                tracing::warn!("{} unreachable: {}", provider, err);
                last_error = ShortenError::Unreachable(err);
            }
        }
    }
    Err(last_error)
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
    fn public_urls_prefer_providers_without_interstitial() {
        // da.gd zeigt für Links unter einer Stunde eine Zwischenseite — für
        // geteilte Links also praktisch immer. Es darf deshalb nur die
        // Rückfallebene sein, nie der erste Anbieter.
        let chain = Provider::chain_for("https://www.declarino.ch/?a=1");
        assert_eq!(chain[0], Provider::IsGd);
        assert_eq!(chain[1], Provider::VGd);
        assert_eq!(*chain.last().unwrap(), Provider::DaGd);
    }

    #[test]
    fn local_urls_use_only_da_gd() {
        // is.gd/v.gd antworten für localhost mit "Please enter a valid URL";
        // sie zu fragen kostet nur Zeit und produziert Fehlermeldungen.
        let chain = Provider::chain_for("http://localhost:8080/app/?a=1");
        assert_eq!(chain, &[Provider::DaGd]);
    }

    #[test]
    fn recognises_non_public_hosts() {
        for local in [
            "http://localhost:8080/app/?a=1",
            "http://127.0.0.1:8080/",
            "http://192.168.1.20/label",
            "http://10.0.0.5/",
            "http://172.16.4.1/",
            "http://nas.local/x",
            "http://buildserver/x",
        ] {
            assert!(!is_publicly_reachable(local), "should be local: {}", local);
        }
        for public in [
            "https://www.declarino.ch/?a=1",
            "https://bar9.github.io/open-farming-hackdays-label-creator/",
            "http://172.32.0.1/",
        ] {
            assert!(is_publicly_reachable(public), "should be public: {}", public);
        }
    }

    #[test]
    fn request_urls_are_encoded() {
        for provider in [Provider::IsGd, Provider::VGd, Provider::DaGd] {
            let url = provider.request_url("https://declarino.ch/?a=1&b=2");
            assert!(url.contains(provider.host()), "wrong host: {}", url);
            assert!(url.contains("%26b%3D2"), "must be encoded: {}", url);
        }
    }

    #[test]
    fn parses_isgd_json_and_dagd_plaintext() {
        assert_eq!(
            Provider::IsGd
                .parse_response("{ \"shorturl\": \"https://is.gd/abc123\" }")
                .unwrap(),
            "https://is.gd/abc123"
        );
        // da.gd antwortet mit einer Zeile Klartext (inkl. Zeilenumbruch).
        assert_eq!(
            Provider::DaGd.parse_response("https://da.gd/YraW\n").unwrap(),
            "https://da.gd/YraW"
        );
    }

    #[test]
    fn rejects_error_text_instead_of_putting_it_in_the_field() {
        assert_eq!(
            Provider::IsGd
                .parse_response("{ \"errorcode\": 1, \"errormessage\": \"Please enter a valid URL\" }")
                .unwrap_err(),
            ShortenError::ProviderRejected("Please enter a valid URL".into())
        );
        assert!(Provider::IsGd
            .parse_response("Error, database insert failed")
            .is_err());
        for body in ["Error: Invalid Url!", "", "http://da.gd/abc"] {
            assert!(
                Provider::DaGd.parse_response(body).is_err(),
                "must reject: {:?}",
                body
            );
        }
    }

    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        // Minimaler Executor: die Kette enthält keine echten Wakeups, alle
        // Test-Futures sind sofort fertig.
        use std::task::{Context, Poll, Waker};
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut fut = Box::pin(fut);
        loop {
            if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
                return out;
            }
        }
    }

    #[test]
    fn public_url_is_shortened_by_isgd() {
        let result = block_on(shorten_with("https://declarino.ch/?a=1", |url| async move {
            assert!(url.contains("is.gd"), "is.gd must be tried first: {}", url);
            Ok("{ \"shorturl\": \"https://is.gd/abc123\" }".to_string())
        }))
        .unwrap();
        assert_eq!(result.provider, Provider::IsGd);
    }

    #[test]
    fn falls_back_to_da_gd_when_both_gd_services_fail() {
        // Realer Ausfall vom 2026-08-27: is.gd und v.gd antworteten beide mit
        // HTTP 200 und "Error, database insert failed".
        let result = block_on(shorten_with("https://declarino.ch/?a=1", |url| async move {
            if url.contains("da.gd") {
                Ok("https://da.gd/abc123".to_string())
            } else {
                Ok("Error, database insert failed".to_string())
            }
        }))
        .unwrap();
        assert_eq!(result.provider, Provider::DaGd);
        assert_eq!(result.url, "https://da.gd/abc123");
    }

    #[test]
    fn localhost_is_shortened_without_asking_the_gd_services() {
        let result = block_on(shorten_with(
            "http://localhost:8080/open-farming-hackdays-label-creator/?a=1",
            |url| async move {
                assert!(url.starts_with("https://da.gd/"), "unexpected call: {}", url);
                assert!(url.contains("localhost"), "localhost must be forwarded");
                Ok("https://da.gd/YraW".to_string())
            },
        ))
        .unwrap();
        assert_eq!(result.provider, Provider::DaGd);
        assert_eq!(result.url, "https://da.gd/YraW");
    }

    #[test]
    fn reports_failure_when_every_provider_is_blocked() {
        let err = block_on(shorten_with("https://declarino.ch/?a=1", |_| async move {
            Err("blocked".to_string())
        }))
        .unwrap_err();
        assert_eq!(err, ShortenError::Unreachable("blocked".into()));
    }
}
