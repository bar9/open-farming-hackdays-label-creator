//! HTTP-Tests des Kurz-Link-Dienstes, portiert aus dem früheren
//! `selftest.mjs`. Laufen gegen den echten Router mit einer flüchtigen
//! Datenbank, also ohne Netz und ohne Server.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use shortener::rules::code_for_url;
use shortener::storage::Store;
use tower::ServiceExt;

const ORIGIN: &str = "https://www.declarino.ch";
const RECIPE: &str = "https://www.declarino.ch/lebensmittelrecht?v=2&product_title=Brownies&ingredients[0][name]=Butter";

fn app() -> (Router, Arc<Store>) {
    let store = Arc::new(Store::open_in_memory().unwrap());
    (shortener::router(store.clone()), store)
}

async fn send(app: &Router, request: Request<Body>) -> Response<Body> {
    app.clone().oneshot(request).await.unwrap()
}

fn post(origin: Option<&str>, body: impl Into<Body>) -> Request<Body> {
    let mut builder = Request::post("/api/shorten").header("content-type", "application/json");
    if let Some(origin) = origin {
        builder = builder.header("origin", origin);
    }
    builder.body(body.into()).unwrap()
}

fn shorten_body(url: &str) -> String {
    serde_json::json!({ "url": url }).to_string()
}

async fn json(response: Response<Body>) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn text(response: Response<Body>) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn header<'a>(response: &'a Response<Body>, name: &str) -> Option<&'a str> {
    response.headers().get(name).and_then(|v| v.to_str().ok())
}

async fn get(app: &Router, path: &str) -> Response<Body> {
    send(app, Request::get(path).body(Body::empty()).unwrap()).await
}

/// 1.-3. Der Normalfall: kürzen, wieder auflösen, zweimal teilen ergibt
/// denselben Code.
#[tokio::test]
async fn shorten_and_resolve() {
    let (app, store) = app();
    let created = send(&app, post(Some(ORIGIN), shorten_body(RECIPE))).await;
    assert_eq!(created.status(), StatusCode::OK);
    assert_eq!(
        header(&created, "access-control-allow-origin"),
        Some(ORIGIN)
    );
    assert_eq!(header(&created, "vary"), Some("Origin"));
    assert_eq!(
        header(&created, "access-control-allow-methods"),
        Some("POST, OPTIONS")
    );
    assert_eq!(
        header(&created, "access-control-allow-headers"),
        Some("Content-Type")
    );
    let body = json(created).await;
    let code = body["code"].as_str().unwrap().to_string();
    assert_eq!(code.len(), 7);
    assert!(code.bytes().all(|b| b.is_ascii_alphanumeric()));
    assert_eq!(code, code_for_url(RECIPE, 7));
    assert_eq!(
        body["short_url"].as_str().unwrap(),
        format!("https://www.declarino.ch/s/{code}")
    );

    let hop = get(&app, &format!("/s/{code}")).await;
    assert_eq!(hop.status(), StatusCode::MOVED_PERMANENTLY);
    assert_eq!(header(&hop, "location"), Some(RECIPE));
    assert_eq!(header(&hop, "cache-control"), Some("public, max-age=86400"));
    assert!(text(hop).await.is_empty(), "301 ohne Body/Zwischenseite");

    // Der alte öffentliche Pfad /api/redirect bleibt gültig.
    let legacy = get(&app, &format!("/api/redirect?code={code}")).await;
    assert_eq!(legacy.status(), StatusCode::MOVED_PERMANENTLY);
    assert_eq!(header(&legacy, "location"), Some(RECIPE));

    let again = json(send(&app, post(Some(ORIGIN), shorten_body(RECIPE))).await).await;
    assert_eq!(again["code"].as_str().unwrap(), code);
    assert_eq!(store.count().unwrap(), 1, "kein zweiter Eintrag");

    // 12. Der Bestand muss für die Sicherung auflistbar sein.
    let all = store.list_all().unwrap();
    assert_eq!(all[0].code, code);
    assert_eq!(all[0].url, RECIPE);
}

/// Die App (src/services/url_shortener.rs) liest genau dieses Format.
#[tokio::test]
async fn response_shape_matches_spoo_me() {
    let (app, _) = app();
    let body = json(send(&app, post(Some(ORIGIN), shorten_body(RECIPE))).await).await;
    let keys: Vec<&str> = body
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"code") && keys.contains(&"short_url"));
}

/// 4. Fremde Ziele müssen abgelehnt werden (Missbrauchsschutz).
#[tokio::test]
async fn foreign_targets_rejected() {
    let (app, store) = app();
    for bad in [
        "https://example.com/phishing",
        "https://www.declarino.ch.evil.com/x",
        "javascript:alert(1)",
        "notaurl",
    ] {
        let res = send(&app, post(Some(ORIGIN), shorten_body(bad))).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(json(res).await["error"], "Nur declarino.ch-Adressen");
    }
    assert_eq!(store.count().unwrap(), 0);
}

/// 5. Staging und localhost müssen den Endpunkt benutzen dürfen.
/// 7. Staging-URLs sind erlaubte Ziele.
#[tokio::test]
async fn staging_and_localhost_allowed() {
    let (app, _) = app();
    for origin in ["https://bar9.github.io", "http://localhost:8080"] {
        let url = format!("{ORIGIN}/lebensmittelrecht?from={origin}");
        let res = send(&app, post(Some(origin), shorten_body(&url))).await;
        assert_eq!(res.status(), StatusCode::OK, "{origin}");
        assert_eq!(header(&res, "access-control-allow-origin"), Some(origin));
    }
    let res = send(
        &app,
        post(
            Some("https://bar9.github.io"),
            shorten_body("https://bar9.github.io/open-farming-hackdays-label-creator/?v=2"),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
}

/// 6. Fremde Herkunft darf nicht durchkommen, und bekommt keine CORS-Freigabe.
#[tokio::test]
async fn foreign_origin_rejected() {
    let (app, store) = app();
    let res = send(
        &app,
        post(Some("https://evil.example"), shorten_body(RECIPE)),
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(header(&res, "access-control-allow-origin"), None);
    assert_eq!(header(&res, "vary"), None);
    assert_eq!(json(res).await["error"], "Herkunft nicht erlaubt");
    assert_eq!(store.count().unwrap(), 0);
}

/// Direkte Aufrufe ohne Origin (curl, Server) sind erlaubt.
#[tokio::test]
async fn no_origin_allowed() {
    let (app, _) = app();
    let res = send(&app, post(None, shorten_body(RECIPE))).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(header(&res, "access-control-allow-origin"), None);
}

/// 8. Preflight muss ohne Body durchgehen.
#[tokio::test]
async fn preflight() {
    let (app, _) = app();
    let res = send(
        &app,
        Request::options("/api/shorten")
            .header("origin", ORIGIN)
            .header("access-control-request-method", "POST")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert_eq!(header(&res, "access-control-allow-origin"), Some(ORIGIN));
}

#[tokio::test]
async fn only_post() {
    let (app, _) = app();
    let res = get(&app, "/api/shorten").await;
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(json(res).await["error"], "Nur POST");
}

/// 9. Unbekannte und unsinnige Codes: 404 statt Fehler.
#[tokio::test]
async fn unknown_codes_404() {
    let (app, _) = app();
    for path in [
        "/s/nichtvorhanden",
        "/s/..%2Fetc%2Fpasswd",
        "/s/zz",
        "/s/",
        "/api/redirect",
        "/api/redirect?code=",
        "/api/redirect?code=zz",
    ] {
        let res = get(&app, path).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{path}");
    }
    let res = get(&app, "/s/nichtvorhanden").await;
    assert_eq!(text(res).await, "Unbekannter Link");
}

/// 10. Zu lange URLs abweisen, damit der Speicher nicht vollläuft.
#[tokio::test]
async fn size_limits() {
    let (app, store) = app();
    let huge = format!("{ORIGIN}/lebensmittelrecht?x={}", "a".repeat(17_000));
    let res = send(&app, post(Some(ORIGIN), shorten_body(&huge))).await;
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(json(res).await["error"], "URL zu lang");

    // Genau an der Grenze geht es noch.
    let prefix = format!("{ORIGIN}/?x=");
    let exact = format!("{prefix}{}", "a".repeat(16 * 1024 - prefix.len()));
    let res = send(&app, post(Some(ORIGIN), shorten_body(&exact))).await;
    assert_eq!(res.status(), StatusCode::OK);

    // Ein Body über der Transportgrenze wird gar nicht erst gelesen.
    let giant = format!("{ORIGIN}/?x={}", "a".repeat(100_000));
    let res = send(&app, post(Some(ORIGIN), shorten_body(&giant))).await;
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(header(&res, "access-control-allow-origin"), Some(ORIGIN));
    assert_eq!(store.count().unwrap(), 1);
}

#[tokio::test]
async fn malformed_bodies() {
    let (app, _) = app();
    let cases: [(&str, &str); 5] = [
        ("{nope", "Ungültiges JSON"),
        ("", "Feld 'url' fehlt"),
        ("{}", "Feld 'url' fehlt"),
        (r#"{"url":""}"#, "Feld 'url' fehlt"),
        (r#"{"url":42}"#, "Feld 'url' fehlt"),
    ];
    for (body, message) in cases {
        let res = send(&app, post(Some(ORIGIN), body.to_string())).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(json(res).await["error"], message, "{body}");
    }
    // Anderer Content-Type: der Body wird trotzdem als JSON gelesen.
    let res = send(
        &app,
        Request::post("/api/shorten")
            .header("content-type", "text/plain")
            .body(Body::from(shorten_body(RECIPE)))
            .unwrap(),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
}

/// Kollision: belegt ein fremder Eintrag den 7er-Code, wächst der Code,
/// statt den Eintrag zu überschreiben.
#[tokio::test]
async fn collision_grows_code() {
    let (app, store) = app();
    let seven = code_for_url(RECIPE, 7);
    store
        .store_if_absent(&seven, "https://www.declarino.ch/fremd")
        .unwrap();
    let body = json(send(&app, post(Some(ORIGIN), shorten_body(RECIPE))).await).await;
    assert_eq!(body["code"].as_str().unwrap(), code_for_url(RECIPE, 8));
    assert_eq!(
        store.lookup(&seven).unwrap().as_deref(),
        Some("https://www.declarino.ch/fremd")
    );
}

/// Ersatz für routecases.json: Was der Dienst selbst beantwortet. Das
/// Zusammenspiel mit Caddy (SPA-Fallback, statische Dateien) prüft
/// ops/caddy/test-routing.sh.
#[tokio::test]
async fn route_cases() {
    let (app, store) = app();
    store
        .store_if_absent("UWKJ6Pe", "https://www.declarino.ch/a")
        .unwrap();
    store
        .store_if_absent("bqoIJ4I", "https://www.declarino.ch/b")
        .unwrap();
    for (path, status) in [
        ("/s/UWKJ6Pe", StatusCode::MOVED_PERMANENTLY),
        ("/s/bqoIJ4I", StatusCode::MOVED_PERMANENTLY),
        ("/api/redirect?code=UWKJ6Pe", StatusCode::MOVED_PERMANENTLY),
        ("/api/shorten", StatusCode::METHOD_NOT_ALLOWED),
        ("/api/unbekannt", StatusCode::NOT_FOUND),
        ("/", StatusCode::NOT_FOUND),
    ] {
        assert_eq!(get(&app, path).await.status(), status, "{path}");
    }
}
