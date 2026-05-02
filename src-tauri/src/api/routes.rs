//! axum route definitions.
//!
//! P0 only exposes a single health-check endpoint. P4 may add read-only endpoints for
//! result/template lookup, either through a dedicated sqlx pool or by reusing the
//! plugin-sql pool internally.

use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct Health {
    ok: bool,
    version: &'static str,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(state)
}

async fn healthz() -> Json<Health> {
    Json(Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
    })
}
