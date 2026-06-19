//! CORS policy for the HTTP API.
//!
//! Two postures, picked by the host:
//! - [`permissive`] — any origin/method/header. Used by `apps/desktop`, whose server is
//!   bound to `127.0.0.1` and only reachable by the local Tauri webview.
//! - [`allow_list`] — an explicit origin allow-list for `apps/server` (Server mode, master
//!   plan §6.6). Never `Any` when the socket is exposed on `0.0.0.0`.

use axum::http::{header, HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

pub fn permissive() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// CORS layer restricted to an explicit list of origins.
///
/// Origins that do not parse as a `HeaderValue` are skipped (the host logs them at the
/// call site). Methods are limited to what the `/v1/` surface uses (`GET`/`POST` plus the
/// `OPTIONS` preflight), and only the `Authorization` + `Content-Type` request headers are
/// allowed so the bearer token and JSON bodies pass through.
pub fn allow_list(origins: &[String]) -> CorsLayer {
    let parsed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(parsed)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}
