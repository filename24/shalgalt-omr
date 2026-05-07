//! OMR computer-vision pipeline.
//!
//! Layering:
//! - `pdf` — pdfium-render rasterization of multi-page PDFs (P3).
//! - `preview` — single-page rasterization used by the P1 template editor.
//! - `perspective` — 4-marker detection + `warpPerspective` (P3).
//! - `bubbles` — per-bubble fill-ratio reading (P3).
//! - `pipeline` — orchestrator + `TaskProgress` / `TaskStage` payloads.
//!
//! Rule 1 / Rule 2 are honored at the crate boundary: every public function takes
//! filesystem paths (never byte buffers crossing IPC) and reports progress via a
//! `tokio::sync::mpsc::Sender<TaskProgress>` rather than blocking the caller.

pub mod bubbles;
pub mod pdf;
pub mod perspective;
pub mod pipeline;
pub mod preview;

pub use pipeline::{TaskProgress, TaskStage};

use std::path::Path;

use shalgalt_core::domain::OmrTemplate;
use shalgalt_core::error::{AppError, AppResult};
use tokio::sync::mpsc;

/// End-to-end OMR processing facade — split a PDF into pages, align each one
/// against the four template markers, read the bubbles, and stream progress
/// events through `progress_tx` while the work happens.
///
/// **Status (P2-03):** signature-only stub. The real implementation lands in
/// P3-01..P3-04 (ArUco markers + adaptive thresholding + auto-deskew +
/// per-bubble confidence). Calling it today returns
/// [`AppError::Internal`] with a clear "not implemented yet" message so the
/// frontend can surface a graceful Mongolian toast instead of crashing.
pub async fn process_pdf(
    _pdf_path: &Path,
    _template: &OmrTemplate,
    _progress_tx: mpsc::Sender<TaskProgress>,
) -> AppResult<Vec<()>> {
    Err(AppError::Internal(anyhow::anyhow!(
        "shalgalt_cv::process_pdf is not implemented yet (P3-01..P3-04)"
    )))
}
