//! Pure-Rust workspace core. No `tauri` dependency — `apps/desktop` and
//! `apps/server` (P5) share these modules.
//!
//! Layering:
//! - `domain` — serialization unit (Rule 3). Mirrored 1:1 in TypeScript.
//! - `grading` — pure `(template, parsed_sheet, answer_key) -> graded_sheet` engine.
//! - `export` — file-format outputs (xlsx today; PDF lives in the sibling `shalgalt-pdf` crate).
//! - `api` — axum `Router` builder + CORS layer. `apps/desktop` owns the spawn (Rule 4).
//! - `error` — `AppError` / `AppResult` envelope shared with the IPC layer.

pub mod api;
pub mod domain;
pub mod error;
pub mod export;
pub mod grading;
pub mod student_id;

pub use student_id::decode_student_id;
