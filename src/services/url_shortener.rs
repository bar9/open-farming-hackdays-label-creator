//! URL-Shortener mit mehreren Anbietern und Fallback-Kette.
//!
//! Hintergrund: die Kurz-Link-Funktion hing an einem einzigen Anbieter
//! (tinyurl). In einigen Netzen (Firmen-Proxies, DNS-Filter, Werbeblocker)
//! ist tinyurl.com blockiert — der Browser meldet dann einen CORS-/Netzwerk-
//! Fehler und der Button war schlicht kaputt. Statt einen Anbieter gegen
//! einen anderen zu tauschen, probieren wir mehrere nacheinander durch: der
//! erste, der antwortet, gewinnt. Alle hier gelisteten Endpunkte senden
//! `Access-Control-Allow-Origin` und sind ohne API-Key nutzbar.

use std::fmt;

/// Ein Anbieter der Fallback-Kette.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Provider {
    /// is.gd — `Access-Control-Allow-Origin: *`, JSON-Antwort.
    IsGd,
    /// v.gd — gleiche Software/API wie is.gd, aber eigene Domain: fällt nicht
    /// unter dieselben Sperrlisten.
    VGd,
    /// spoo.me — POST-API mit `Access-Control-Allow-Origin: *`. Unabhängige
    /// Infrastruktur, springt ein wenn is.gd/v.gd gemeinsam ausfallen (beide
    /// laufen auf derselben Software und antworten dann mit
    /// "Error, database insert failed").
    Spoo,
    /// tinyurl — historischer Anbieter, als letzte Station behalten.
    TinyUrl,
}

impl Provider {
    /// Reihenfolge der Versuche. is.gd/v.gd zuerst, weil sie `ACAO: *` senden
    /// und seltener gefiltert werden als tinyurl.
    pub const CHAIN: [Provider; 4] = [
        Provider::IsGd,
        Provider::VGd,
        Provider::Spoo,
        Provider::TinyUrl,
    ];

    pub fn host(&self) -> &'static str {
        match self {
            Provider::IsGd => "is.gd",
            Provider::VGd => "v.gd",
            Provider::Spoo => "spoo.me",
            Provider::TinyUrl => "tinyurl.com",
        }
    }

    /// Anfrage-URL für die zu kürzende Adresse.
    pub fn request_url(&self, long_url: &str) -> String {
        let encoded = urlencoding::encode(long_url);
        match self {
            Provider::IsGd => format!("https://is.gd/create.php?format=json&url={}", encoded),
            Provider::VGd => format!("https://v.gd/create.php?format=json&url={}", encoded),
            Provider::Spoo => "https://spoo.me/".to_string(),
            Provider::TinyUrl => format!("https://tinyurl.com/api-create.php?url={}", encoded),
        }
    }

    /// Formular-Body für POST-Anbieter; `None` bedeutet GET.
    pub fn request_body(&self, long_url: &str) -> Option<String> {
        match self {
            Provider::Spoo => Some(format!("url={}", urlencoding::encode(long_url))),
            _ => None,
        }
    }

    /// Antwort des Anbieters in einen Kurz-Link übersetzen.
    ///
    /// Wichtig: mehrere dieser Dienste antworten mit HTTP 200 und einer
    /// Fehlermeldung im Body ("Error, database insert failed",
    /// `{"errorcode":1,...}`). Ohne Prüfung landete so ein Fehlertext im
    /// Eingabefeld — deshalb wird jede Antwort validiert.
    pub fn parse_response(&self, body: &str) -> Result<String, ShortenError> {
        let body = body.trim();
        let candidate = match self {
            Provider::IsGd | Provider::VGd => extract_json_shorturl(body)?,
            Provider::Spoo => force_https(&extract_json_field(body, "short_url")?),
            Provider::TinyUrl => body.to_string(),
        };
        if is_plausible_short_url(&candidate) {
            Ok(candidate)
        } else {
            Err(ShortenError::ProviderRejected(candidate.to_string()))
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
    /// Die Adresse ist nur lokal erreichbar (localhost, 127.0.0.1, .local).
    /// Kein öffentlicher Shortener kann darauf zeigen — und ein Kurz-Link auf
    /// `localhost` wäre für Empfänger ohnehin wertlos.
    NotPubliclyReachable(String),
}

impl fmt::Display for ShortenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShortenError::ProviderRejected(msg) => write!(f, "provider rejected: {}", msg),
            ShortenError::Unreachable(msg) => write!(f, "unreachable: {}", msg),
            ShortenError::AllProvidersFailed => f.write_str("all providers failed"),
            ShortenError::NotPubliclyReachable(host) => {
                write!(f, "not publicly reachable: {}", host)
            }
        }
    }
}

/// Host einer URL grob extrahieren (ohne url-Crate, reicht für die Prüfung).
fn host_of(url: &str) -> String {
    let rest = url
        .split_once("://")
        .map(|(_, r)| r)
        .unwrap_or(url);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = authority.rsplit_once('@').map(|(_, h)| h).unwrap_or(authority);
    let host = host.split_once(':').map(|(h, _)| h).unwrap_or(host);
    host.trim_matches(['[', ']']).to_lowercase()
}

/// Nur öffentlich auflösbare Adressen lassen sich sinnvoll kürzen.
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

/// Ergebnis einer erfolgreichen Kürzung — inklusive Anbieter, damit die UI
/// transparent machen kann, wo das Rezept nun liegt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortLink {
    pub url: String,
    pub provider: Provider,
}

fn extract_json_shorturl(body: &str) -> Result<String, ShortenError> {
    extract_json_field(body, "shorturl")
}

fn extract_json_field(body: &str, field: &str) -> Result<String, ShortenError> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| ShortenError::ProviderRejected(body.to_string()))?;
    if let Some(short) = value.get(field).and_then(|v| v.as_str()) {
        return Ok(short.to_string());
    }
    let message = value
        .get("errormessage")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("error").and_then(|v| v.as_str()))
        .unwrap_or(body);
    Err(ShortenError::ProviderRejected(message.to_string()))
}

/// spoo.me liefert `http://spoo.me/...`; auf einer https-Seite wäre das ein
/// Mixed-Content-Link. Die Domain kann https, also heben wir das Schema an.
fn force_https(url: &str) -> String {
    match url.strip_prefix("http://") {
        Some(rest) => format!("https://{}", rest),
        None => url.to_string(),
    }
}

/// Grobe Plausibilitätsprüfung: eine kurze https-URL ohne Leerzeichen. Hält
/// Fehlertexte ("Error, database insert failed") aus dem Ergebnisfeld heraus.
fn is_plausible_short_url(candidate: &str) -> bool {
    candidate.starts_with("https://")
        && !candidate.contains(char::is_whitespace)
        && candidate.len() > "https://a.b/c".len()
        && candidate.len() < 200
}

/// Kürzt `long_url`, indem die Anbieter der Reihe nach probiert werden.
/// `fetch` kapselt den HTTP-Aufruf (URL, optionaler Formular-Body für
/// POST-Anbieter), damit die Kettenlogik ohne Netz und ohne wasm testbar
/// bleibt.
pub async fn shorten_with<F, Fut>(long_url: &str, fetch: F) -> Result<ShortLink, ShortenError>
where
    F: Fn(String, Option<String>) -> Fut,
    Fut: std::future::Future<Output = Result<String, String>>,
{
    // Öffentliche Shortener lehnen localhost & Co. ab (is.gd/v.gd/spoo.me mit
    // "invalid URL"); nur tinyurl kürzt sie klaglos — und produziert damit
    // einen Link, der ausserhalb dieses Rechners ins Leere zeigt. Deshalb hier
    // abbrechen statt die Kette bis tinyurl durchlaufen zu lassen.
    if !is_publicly_reachable(long_url) {
        return Err(ShortenError::NotPubliclyReachable(host_of(long_url)));
    }
    let mut last_error = ShortenError::AllProvidersFailed;
    for provider in Provider::CHAIN {
        match fetch(provider.request_url(long_url), provider.request_body(long_url)).await {
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
    shorten_with(long_url, |url, body| async move {
        let response = match body {
            Some(form) => gloo::net::http::Request::post(&url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Accept", "application/json")
                .body(form)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?,
            None => gloo::net::http::Request::get(&url)
                .send()
                .await
                .map_err(|e| e.to_string())?,
        };
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
    fn chain_starts_with_cors_friendly_providers() {
        // tinyurl ist in manchen Netzen gesperrt und darf nicht mehr der
        // einzige/erste Anbieter sein.
        assert_eq!(Provider::CHAIN[0], Provider::IsGd);
        assert!(Provider::CHAIN.contains(&Provider::VGd));
        assert_eq!(*Provider::CHAIN.last().unwrap(), Provider::TinyUrl);
    }

    #[test]
    fn chain_contains_provider_on_independent_infrastructure() {
        // is.gd und v.gd laufen auf derselben Software: fällt deren Datenbank
        // aus ("Error, database insert failed"), scheitern beide gleichzeitig.
        // Vor tinyurl muss deshalb ein unabhängiger Anbieter stehen.
        let isgd = Provider::CHAIN.iter().position(|p| *p == Provider::IsGd).unwrap();
        let spoo = Provider::CHAIN.iter().position(|p| *p == Provider::Spoo).unwrap();
        let tiny = Provider::CHAIN.iter().position(|p| *p == Provider::TinyUrl).unwrap();
        assert!(isgd < spoo && spoo < tiny);
    }

    #[test]
    fn spoo_posts_form_body_and_parses_https_link() {
        assert_eq!(
            Provider::Spoo.request_body("https://declarino.ch/?a=1&b=2").unwrap(),
            "url=https%3A%2F%2Fdeclarino.ch%2F%3Fa%3D1%26b%3D2"
        );
        assert!(Provider::IsGd.request_body("https://declarino.ch/").is_none());
        // spoo.me liefert http:// — auf https-Seiten wäre das Mixed Content.
        assert_eq!(
            Provider::Spoo
                .parse_response("{\"short_url\":\"http://spoo.me/pKaQB9\"}")
                .unwrap(),
            "https://spoo.me/pKaQB9"
        );
        assert!(Provider::Spoo
            .parse_response("{\"error\":\"Invalid URL\"}")
            .is_err());
    }

    #[test]
    fn falls_back_to_spoo_when_isgd_and_vgd_database_fails() {
        // Realer Ausfall vom 2026-08-27: beide gd-Dienste antworteten mit
        // HTTP 200 und "Error, database insert failed".
        let result = block_on(shorten_with(
            "https://declarino.ch/?a=1",
            |url, body| async move {
                if url.contains("spoo.me") {
                    assert!(body.is_some(), "spoo.me must be called via POST");
                    Ok("{\"short_url\":\"http://spoo.me/abc123\"}".to_string())
                } else if url.contains("tinyurl") {
                    Ok("https://tinyurl.com/should-not-be-used".to_string())
                } else {
                    Ok("Error, database insert failed".to_string())
                }
            },
        ))
        .unwrap();
        assert_eq!(result.provider, Provider::Spoo);
        assert_eq!(result.url, "https://spoo.me/abc123");
    }

    #[test]
    fn request_urls_are_encoded() {
        let url = Provider::IsGd.request_url("https://declarino.ch/?a=1&b=2");
        assert!(url.starts_with("https://is.gd/create.php?format=json&url="));
        assert!(url.contains("%26b%3D2"), "query must be percent-encoded: {}", url);
    }

    #[test]
    fn parses_isgd_json() {
        let parsed = Provider::IsGd.parse_response("{ \"shorturl\": \"https://is.gd/abc123\" }");
        assert_eq!(parsed.unwrap(), "https://is.gd/abc123");
    }

    #[test]
    fn rejects_isgd_error_payload() {
        let err = Provider::IsGd
            .parse_response("{ \"errorcode\": 1, \"errormessage\": \"Please enter a valid URL\" }")
            .unwrap_err();
        assert_eq!(err, ShortenError::ProviderRejected("Please enter a valid URL".into()));
    }

    #[test]
    fn rejects_plaintext_error_from_tinyurl() {
        // tinyurl antwortet mit HTTP 200 und Fehlertext im Body.
        assert!(Provider::TinyUrl.parse_response("Error").is_err());
        assert!(Provider::IsGd.parse_response("Error, database insert failed").is_err());
    }

    #[test]
    fn accepts_tinyurl_plain_url() {
        assert_eq!(
            Provider::TinyUrl.parse_response("https://tinyurl.com/2abcde\n").unwrap(),
            "https://tinyurl.com/2abcde"
        );
    }

    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        // Minimaler Executor: die Kette enthält keine echten Wakeups, alle
        // Test-Futures sind sofort fertig.
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
    fn falls_back_to_next_provider_when_first_is_blocked() {
        let result = block_on(shorten_with("https://declarino.ch/?a=1", |url, _body| async move {
            if url.contains("is.gd/create") && !url.contains("v.gd") {
                // Simuliert die CORS-/Netzwerksperre.
                Err("NetworkError: Failed to fetch".to_string())
            } else if url.contains("v.gd") {
                Ok("{ \"shorturl\": \"https://v.gd/xyz789\" }".to_string())
            } else {
                Ok("https://tinyurl.com/should-not-be-used".to_string())
            }
        }))
        .unwrap();
        assert_eq!(result.provider, Provider::VGd);
        assert_eq!(result.url, "https://v.gd/xyz789");
    }

    #[test]
    fn falls_through_to_tinyurl_as_last_resort() {
        let result = block_on(shorten_with("https://declarino.ch/?a=1", |url, _body| async move {
            if url.contains("tinyurl") {
                Ok("https://tinyurl.com/abc999".to_string())
            } else {
                Err("blocked".to_string())
            }
        }))
        .unwrap();
        assert_eq!(result.provider, Provider::TinyUrl);
    }

    #[test]
    fn reports_failure_when_every_provider_is_blocked() {
        let err = block_on(shorten_with("https://declarino.ch/?a=1", |_, _| async move {
            Err("blocked".to_string())
        }))
        .unwrap_err();
        assert_eq!(err, ShortenError::Unreachable("blocked".into()));
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
    fn localhost_is_not_shortened_via_tinyurl() {
        // tinyurl kürzt localhost-Links klaglos — das Ergebnis zeigt für
        // Empfänger aber ins Leere. Erst gar nicht anfragen.
        let err = block_on(shorten_with(
            "http://localhost:8080/open-farming-hackdays-label-creator/?a=1",
            |url, _| async move {
                panic!("no provider must be contacted, but got {}", url);
                #[allow(unreachable_code)]
                Err::<String, String>("unreachable".to_string())
            },
        ))
        .unwrap_err();
        assert_eq!(err, ShortenError::NotPubliclyReachable("localhost".into()));
    }
}
