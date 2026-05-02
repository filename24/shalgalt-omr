//! Rule 4 — axum HTTP server that runs independently of the Tauri UI thread.

pub mod cors;
pub mod routes;
pub mod server;

pub use server::{spawn, ApiHandle};
