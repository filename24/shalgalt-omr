//! Rule 4 — Tauri UI 스레드와 독립적으로 동작하는 axum 백그라운드 HTTP 서버.

pub mod cors;
pub mod routes;
pub mod server;

pub use server::{spawn, ApiHandle};
