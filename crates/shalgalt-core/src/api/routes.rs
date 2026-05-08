//! axum route definitions.
//!
//! P0 only exposes a single health-check endpoint. P5 grows this into the `/v1/`
//! REST surface (see master plan §6.6). The router is intentionally stateless so
//! the same `Router` can be embedded in either `apps/desktop` (Tauri host) or
//! `apps/server` (standalone binary). Endpoints that need shared state should
//! adopt axum's typed-state pattern when they land — keep the surface uniform.

use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    ok: bool,
    version: &'static str,
}

pub fn router() -> Router {
    Router::new().route("/healthz", get(healthz))
}

async fn healthz() -> Json<Health> {
    Json(Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
    })
}
