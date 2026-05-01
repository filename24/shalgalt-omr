//! axum 라우트 정의.
//!
//! P0에서는 헬스 체크 1개만 노출한다.
//! P4에서 결과/템플릿 외부 노출이 필요해지면 별도 sqlx 풀(또는 tauri-plugin-sql 의 내부 풀 공유)을 통해
//! read-only 엔드포인트를 추가한다.

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
