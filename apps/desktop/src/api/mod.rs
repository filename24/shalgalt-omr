//! Rule 4 — process-level harness for the axum HTTP server.
//!
//! The router and CORS layer live in `shalgalt_core::api`; this module owns the
//! `tokio::spawn` task, port binding, and oneshot graceful-shutdown signal so the
//! server's lifecycle is tied to the host process (Tauri here, `apps/server` in P5).

pub mod server;

pub use server::{spawn, ApiHandle};
