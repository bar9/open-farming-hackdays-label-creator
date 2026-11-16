//! Kurz-Link-Dienst für declarino.ch.
//!
//! ```text
//! POST /api/shorten   { "url": "https://www.declarino.ch/..." }
//!                  -> 200 { "code": "AbC1234", "short_url": "https://www.declarino.ch/s/AbC1234" }
//! GET  /s/:code    -> 301 auf die hinterlegte Adresse
//! ```
//!
//! Antwortformat und Feldname `short_url` stammen von spoo.me, damit die
//! Rust-Seite der App (`src/services/url_shortener.rs`) denselben Parser
//! verwendet. Statuscodes und Fehlermeldungen sind bewusst identisch mit dem
//! früheren JavaScript-Dienst.

pub mod rules;
pub mod storage;

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use serde_json::{Value, json};

use rules::{
    CODE_LENGTH, MAX_CODE_LENGTH, MAX_URL_LENGTH, SHORT_BASE, code_for_url, is_allowed_origin,
    is_allowed_target, is_valid_code, js_length,
};
use storage::Store;

/// Obergrenze für den ganzen Request-Body. Grösser als `MAX_URL_LENGTH`
/// (JSON-Hülle, Escapes), aber klein genug, dass niemand den Speicher des
/// Dienstes mit Riesen-Bodies füllt. Darüber: 413 wie bei zu langer URL.
pub const MAX_BODY_BYTES: usize = 64 * 1024;

pub type AppState = Arc<Store>;

pub fn router(store: AppState) -> Router {
    Router::new()
        .route("/api/shorten", any(shorten))
        .route("/s/{code}", any(redirect_path))
        // Früher über den Rewrite /s/:code -> /api/redirect?code=… erreichbar
        // und damit öffentlich. Bleibt, damit kein bestehender Aufruf bricht.
        .route("/api/redirect", any(redirect_query))
        .fallback(not_found)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(store)
}

fn json_response(status: StatusCode, body: Value) -> Response {
    (status, axum::Json(body)).into_response()
}

fn error(status: StatusCode, message: &str) -> Response {
    json_response(status, json!({ "error": message }))
}

/// CORS-Header setzen. Ohne diese ist der Endpunkt aus dem WASM-Frontend
/// nicht aufrufbar; daran sind mehrere Fremddienste gescheitert.
fn apply_cors(origin: Option<&str>, response: &mut Response) {
    let headers = response.headers_mut();
    if let Some(origin) = origin.and_then(|o| HeaderValue::from_str(o).ok()) {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        // Antwort hängt von der Origin ab; ohne Vary könnte ein Cache die
        // Freigabe für eine fremde Herkunft ausliefern.
        headers.insert(header::VARY, HeaderValue::from_static("Origin"));
    }
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type"),
    );
}

async fn shorten(
    State(store): State<AppState>,
    method: Method,
    headers: HeaderMap,
    body: Result<Bytes, axum::extract::rejection::BytesRejection>,
) -> Response {
    let origin_header = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    let allowed_origin = origin_header.filter(|o| is_allowed_origin(o));
    let mut response = shorten_inner(&store, &method, origin_header, allowed_origin, body);
    apply_cors(allowed_origin, &mut response);
    response
}

fn shorten_inner(
    store: &Store,
    method: &Method,
    origin_header: Option<&str>,
    allowed_origin: Option<&str>,
    body: Result<Bytes, axum::extract::rejection::BytesRejection>,
) -> Response {
    // Preflight: der Browser fragt vor dem eigentlichen POST an.
    if method == Method::OPTIONS {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method != Method::POST {
        return error(StatusCode::METHOD_NOT_ALLOWED, "Nur POST");
    }
    // Ohne erlaubte Origin gar nicht erst arbeiten. Direkte Aufrufe ohne
    // Origin-Header (curl, Server) sind erlaubt: CORS schützt Browser-Nutzer,
    // nicht den Endpunkt selbst.
    if origin_header.is_some() && allowed_origin.is_none() {
        return error(StatusCode::FORBIDDEN, "Herkunft nicht erlaubt");
    }

    let body = match body {
        Ok(body) => body,
        Err(rejection) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            return error(StatusCode::PAYLOAD_TOO_LARGE, "URL zu lang");
        }
        Err(_) => return error(StatusCode::BAD_REQUEST, "Ungültiges JSON"),
    };
    // Der Body wird unabhängig vom Content-Type als JSON gelesen, wie früher:
    // die App schickt application/json, curl-Aufrufe oft nicht.
    let parsed: Value = if body.iter().all(u8::is_ascii_whitespace) {
        Value::Null
    } else {
        match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => return error(StatusCode::BAD_REQUEST, "Ungültiges JSON"),
        }
    };
    let url = match parsed.get("url").and_then(Value::as_str) {
        Some(url) if !url.is_empty() => url,
        _ => return error(StatusCode::BAD_REQUEST, "Feld 'url' fehlt"),
    };
    if js_length(url) > MAX_URL_LENGTH {
        return error(StatusCode::PAYLOAD_TOO_LARGE, "URL zu lang");
    }
    if !is_allowed_target(url) {
        // Kein fremdes Ziel: siehe ALLOWED_TARGET_HOSTS in rules.rs.
        return error(StatusCode::BAD_REQUEST, "Nur declarino.ch-Adressen");
    }

    // Der Code ist der Hash der URL, dieselbe URL ergibt also denselben Link.
    // Bei einer Kollision (gleicher Code, anderes Ziel) wächst der Code, statt
    // den fremden Eintrag zu überschreiben, sonst landete ein bereits
    // verschickter Link stillschweigend woanders.
    for length in CODE_LENGTH..=MAX_CODE_LENGTH {
        let code = code_for_url(url, length);
        let hit = store
            .store_if_absent(&code, url)
            .and_then(|written| Ok(written || store.lookup(&code)?.as_deref() == Some(url)));
        match hit {
            Ok(true) => {
                return json_response(
                    StatusCode::OK,
                    json!({ "code": code, "short_url": format!("{SHORT_BASE}/s/{code}") }),
                );
            }
            Ok(false) => continue,
            Err(e) => {
                eprintln!("shorten failed: {e}");
                return error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Speichern fehlgeschlagen",
                );
            }
        }
    }
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Kein freier Code gefunden",
    )
}

fn text(status: StatusCode, message: &'static str) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        message,
    )
        .into_response()
}

/// Ein echter 301 ohne Zwischenseite ist der ganze Zweck der Übung:
/// Fremddienste blenden für frische Links Warnseiten ein.
fn redirect(store: &Store, code: &str) -> Response {
    if !is_valid_code(code) {
        return text(StatusCode::NOT_FOUND, "Unbekannter Link");
    }
    match store.lookup(code) {
        Ok(Some(target)) => {
            // 301 statt 302: Kurz-Links sind unveränderlich (der Code ist der
            // Hash des Ziels), also darf der Browser das Ergebnis behalten.
            let Ok(location) = HeaderValue::from_str(&target) else {
                eprintln!("ungültige Ziel-URL für {code}");
                return text(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Nachschlagen fehlgeschlagen",
                );
            };
            Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, location)
                .header(header::CACHE_CONTROL, "public, max-age=86400")
                .body(Body::empty())
                .expect("gültige Antwort")
        }
        Ok(None) => text(StatusCode::NOT_FOUND, "Unbekannter Link"),
        Err(e) => {
            eprintln!("lookup failed: {e}");
            text(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Nachschlagen fehlgeschlagen",
            )
        }
    }
}

async fn redirect_path(State(store): State<AppState>, Path(code): Path<String>) -> Response {
    redirect(&store, &code)
}

#[derive(serde::Deserialize)]
struct CodeQuery {
    #[serde(default)]
    code: String,
}

async fn redirect_query(
    State(store): State<AppState>,
    query: Result<Query<CodeQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let code = query.map(|q| q.0.code).unwrap_or_default();
    redirect(&store, &code)
}

async fn not_found() -> Response {
    text(StatusCode::NOT_FOUND, "Nicht gefunden")
}
