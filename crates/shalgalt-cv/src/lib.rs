//! OMR computer-vision pipeline.
//!
//! Layering:
//! - `pdf` — pdfium-render rasterization of multi-page PDFs.
//! - `preview` — single-page rasterization used by the P1 template editor.
//! - `threshold` — adaptive Gaussian thresholding + median-blur background removal.
//! - `deskew` — pre-warp `HoughLinesP` rotation correction.
//! - `perspective` — ArUco DICT_6X6_50 detection + `warpPerspective` to canonical.
//! - `bubbles` — per-bubble fill-ratio reading + confidence scoring.
//! - `pipeline` — orchestrates the above and emits [`TaskProgress`] events.
//!
//! `TaskProgress` / `TaskStage` are part of the IPC payload surface and live in
//! `shalgalt_core::domain::progress`. They are re-exported here so the call
//! sites that previously imported them from `shalgalt_cv` keep compiling.
//!
//! Rule 1 / Rule 2 are honored at the crate boundary: every public function takes
//! filesystem paths (never byte buffers crossing IPC) and reports progress via a
//! `tokio::sync::mpsc::Sender<TaskProgress>` rather than blocking the caller.

pub mod bubbles;
pub mod deskew;
pub mod pdf;
pub mod perspective;
mod pipeline;
pub mod preview;
pub mod threshold;

pub use shalgalt_core::domain::{TaskProgress, TaskStage};

use std::path::Path;

use shalgalt_core::domain::{OmrTemplate, ParsedSheet};
use shalgalt_core::error::AppResult;
use tokio::sync::mpsc;

/// End-to-end OMR processing facade — split a PDF into pages, align each one
/// against the four template ArUco markers, read the bubbles, and stream progress
/// events through `progress_tx` while the work happens.
///
/// `cache_dir` is where intermediate page rasters land. The desktop app passes its
/// `app_cache_dir`; tests typically pass a `tempfile::TempDir`.
pub async fn process_pdf(
    pdf_path: &Path,
    template: &OmrTemplate,
    progress_tx: mpsc::Sender<TaskProgress>,
    task_id: &str,
    cache_dir: &Path,
) -> AppResult<Vec<ParsedSheet>> {
    pipeline::run(
        pdf_path.to_path_buf(),
        template.clone(),
        progress_tx,
        task_id.to_string(),
        cache_dir.to_path_buf(),
    )
    .await
}
