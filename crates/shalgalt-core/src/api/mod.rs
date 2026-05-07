//! Rule 4 — axum HTTP API surface. The `Router` is built here so it is reusable by both
//! `apps/desktop` (foreground Tauri shell) and `apps/server` (P5 standalone binary).
//! Process-level concerns — port binding, oneshot shutdown, tokio task ownership —
//! belong to the consuming app, not to this module.

pub mod cors;
pub mod routes;

pub use routes::router;
