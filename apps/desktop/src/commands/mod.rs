//! Tauri IPC entry points.
//!
//! Each handler is a thin shim: validate input → call domain / CV code → convert to
//! `AppError`.
//!
//! Database CRUD is performed from the frontend through `tauri-plugin-sql`, so this module
//! contains no read/write commands. What lives here is (a) Rust-only heavy work (scan) and
//! (b) external-format conversion (xlsx).

pub mod export;
pub mod pdf;
pub mod scan;
