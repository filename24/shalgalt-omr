//! Rule 4 — axum HTTP API surface. The `Router` is built here so it is reusable by both
//! `apps/desktop` (foreground Tauri shell) and `apps/server` (P5 standalone binary).
//! Process-level concerns — port binding, oneshot shutdown, tokio task ownership, auth and
//! CORS configuration — belong to the consuming app, not to this module.
//!
//! Persistence is reached through the [`DataStore`] trait: core defines the seam and the
//! DTOs, each host injects a concrete (rusqlite) implementation. Core itself never touches
//! a database.

pub mod auth;
pub mod cors;
pub mod dto;
pub mod openapi;
pub mod routes;
pub mod state;
pub mod store;

pub use dto::{
    ExamDto, NewExam, NewResult, ResultDto, ResultFilter, TemplateDto, TemplateSummaryDto,
};
pub use openapi::ApiDoc;
pub use routes::router;
pub use state::AppState;
pub use store::{read_only, DataStore, MemoryStore};

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    fn app(store: Arc<dyn DataStore>) -> axum::Router {
        router(AppState::new(store))
    }

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn healthz_reports_ok() {
        let resp = app(Arc::new(MemoryStore::new()))
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(body_json(resp).await["ok"], serde_json::json!(true));
    }

    #[tokio::test]
    async fn unknown_exam_is_404() {
        let resp = app(Arc::new(MemoryStore::new()))
            .oneshot(Request::get("/v1/exams/999").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_json(resp).await["code"],
            serde_json::json!("not_found")
        );
    }

    #[tokio::test]
    async fn create_then_list_exam_roundtrips() {
        let store = Arc::new(MemoryStore::new());
        store.seed_template("Math", "{}");

        let create = app(store.clone())
            .oneshot(
                Request::post("/v1/exams")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"Final","template_id":1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create.status(), StatusCode::CREATED);
        assert_eq!(body_json(create).await["id"], serde_json::json!(2));

        let list = app(store)
            .oneshot(Request::get("/v1/exams").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(list.status(), StatusCode::OK);
        let json = body_json(list).await;
        assert_eq!(json.as_array().unwrap().len(), 1);
        assert_eq!(json[0]["name"], serde_json::json!("Final"));
    }

    #[tokio::test]
    async fn read_only_store_rejects_writes_with_400() {
        let resp = app(Arc::new(MemoryStore::read_only()))
            .oneshot(
                Request::post("/v1/exams")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"X","template_id":1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn openapi_document_lists_v1_paths() {
        let resp = app(Arc::new(MemoryStore::new()))
            .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert!(json["paths"]["/v1/exams"].is_object());
        assert!(json["paths"]["/v1/results"].is_object());
    }

    #[tokio::test]
    async fn results_filter_by_exam_id() {
        let store = Arc::new(MemoryStore::new());
        for body in [
            r#"{"template_id":1,"exam_id":1,"total_score":10,"detail_answers":"[]"}"#,
            r#"{"template_id":1,"exam_id":2,"total_score":20,"detail_answers":"[]"}"#,
        ] {
            let resp = app(store.clone())
                .oneshot(
                    Request::post("/v1/results")
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);
        }

        let filtered = app(store)
            .oneshot(
                Request::get("/v1/results?exam_id=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let json = body_json(filtered).await;
        assert_eq!(json.as_array().unwrap().len(), 1);
        assert_eq!(json[0]["exam_id"], serde_json::json!(2));
    }
}
