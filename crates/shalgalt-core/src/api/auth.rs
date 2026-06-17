//! Optional bearer-token guard for the `/v1/` surface (P5-04).
//!
//! Core owns the middleware so both hosts share one implementation, but core never reads
//! the environment (Rule 4): the token is passed in by the host. `apps/server` wraps the
//! router with [`with_bearer`] using `SHALGALT_API_TOKEN`; `apps/desktop` binds to
//! `127.0.0.1` and skips auth entirely, so the only reachable client is the local webview
//! (master plan §6.6).

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::{from_fn_with_state, Next},
    response::{IntoResponse, Response},
    Json, Router,
};

use crate::error::AppErrorPayload;

/// Wrap `router` so every request must carry `Authorization: Bearer <token>`.
///
/// The token is moved into the layer's state as an `Arc<str>`; mismatches and missing
/// headers both yield `401 Unauthorized` with the standard `{ code, message }` body.
pub fn with_bearer(router: Router, token: String) -> Router {
    let token: Arc<str> = Arc::from(token);
    router.layer(from_fn_with_state(token, require_bearer))
}

async fn require_bearer(State(token): State<Arc<str>>, req: Request<Body>, next: Next) -> Response {
    let presented = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match presented {
        Some(candidate) if constant_time_eq(candidate.as_bytes(), token.as_bytes()) => {
            next.run(req).await
        }
        _ => unauthorized(),
    }
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer")],
        Json(AppErrorPayload {
            code: "unauthorized",
            message: "missing or invalid bearer token".to_string(),
        }),
    )
        .into_response()
}

/// Length-aware constant-time byte comparison. Avoids leaking the token via early-exit
/// timing on the hot reject path; falls through `false` immediately on a length mismatch
/// (which itself reveals nothing about the secret's contents).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn matches_identical_tokens() {
        assert!(constant_time_eq(b"s3cr3t", b"s3cr3t"));
    }

    #[test]
    fn rejects_different_tokens_and_lengths() {
        assert!(!constant_time_eq(b"s3cr3t", b"s3cr3T"));
        assert!(!constant_time_eq(b"short", b"longer-token"));
    }
}
