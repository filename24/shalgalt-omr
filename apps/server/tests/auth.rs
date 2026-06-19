//! Server-mode auth behavior (P5-04/05). Exercises the assembled app via `tower`'s
//! `oneshot` so no socket is bound.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use shalgalt_core::api::{AppState, MemoryStore};
use shalgalt_server::build_app;
use tower::ServiceExt;

fn state() -> AppState {
    AppState::new(Arc::new(MemoryStore::new()))
}

#[tokio::test]
async fn missing_token_is_401() {
    let app = build_app(state(), &[], Some("secret".to_string()));
    let resp = app
        .oneshot(Request::get("/v1/exams").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_token_is_401() {
    let app = build_app(state(), &[], Some("secret".to_string()));
    let resp = app
        .oneshot(
            Request::get("/v1/exams")
                .header("authorization", "Bearer nope")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn correct_token_is_authorized() {
    let app = build_app(state(), &[], Some("secret".to_string()));
    let resp = app
        .oneshot(
            Request::get("/v1/exams")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"[]");
}

#[tokio::test]
async fn no_token_configured_serves_unauthenticated() {
    let app = build_app(state(), &[], None);
    let resp = app
        .oneshot(Request::get("/v1/exams").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn healthz_requires_auth_too_when_token_set() {
    // The whole router is behind the guard; only the CORS preflight is exempt.
    let app = build_app(state(), &[], Some("secret".to_string()));
    let resp = app
        .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
