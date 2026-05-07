//! Computer-vision grading pipeline.
//!
//! Blueprint Rule 1: inputs are always **local absolute path strings**, and result images
//! are written to a temp directory with only the path returned (no Base64 over IPC).
//! Blueprint Rule 2: every heavy operation runs in a `tokio::spawn` background task and
//! emits progress through `app_handle.emit("task-progress", ...)`.
//!
//! P0 only defines the interfaces. The actual `opencv-rust` and `pdfium-render` calls land
//! in P2.

pub mod bubbles;
pub mod pdf;
pub mod perspective;
pub mod pipeline;
pub mod preview;

pub use pipeline::{TaskProgress, TaskStage};
