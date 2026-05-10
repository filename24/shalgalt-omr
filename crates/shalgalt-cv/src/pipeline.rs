//! End-to-end OMR pipeline orchestrator.
//!
//! Wires the [`pdf`], [`threshold`], [`deskew`], [`perspective`], and [`bubbles`]
//! stages together and emits [`TaskProgress`] events at every stage transition.
//! Synchronous OpenCV / pdfium work runs inside a [`tokio::task::spawn_blocking`]
//! so the caller's runtime stays responsive (Rule 2).

use std::path::{Path, PathBuf};

use opencv::{imgcodecs, prelude::*};

use shalgalt_core::domain::{OmrTemplate, ParsedSheet, TaskProgress, TaskStage};
use shalgalt_core::error::{AppError, AppResult};

use crate::{
    bubbles, deskew, pdf,
    perspective::{self, MarkerLayout},
    threshold,
};

/// Run the full pipeline on `pdf_path`. Emits progress events through `progress_tx`
/// and returns one [`ParsedSheet`] per page.
///
/// This function is the implementation behind the crate-level
/// [`crate::process_pdf`] facade. It is `pub(crate)` because the public surface
/// is the facade — having two entry points named the same thing is confusing.
pub(crate) async fn run(
    pdf_path: PathBuf,
    template: OmrTemplate,
    progress_tx: tokio::sync::mpsc::Sender<TaskProgress>,
    task_id: String,
    cache_dir: PathBuf,
) -> AppResult<Vec<ParsedSheet>> {
    let template_id = derive_template_id(&template);
    let layout = MarkerLayout::from_template(&template.markers);

    // Stage 1: load + rasterize. pdfium is sync.
    emit(&progress_tx, &task_id, 0, 0, TaskStage::LoadingPdf, None).await;

    let raster_dir = cache_dir.join("page-rasters").join(&task_id);
    let pdf_path_for_blocking = pdf_path.clone();
    let raster_dir_clone = raster_dir.clone();
    let page_paths: Vec<PathBuf> = tokio::task::spawn_blocking(move || {
        pdf::rasterize_all_pages(&pdf_path_for_blocking, &raster_dir_clone)
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("rasterize join: {e}")))??;

    let total = page_paths.len();
    if total == 0 {
        emit(
            &progress_tx,
            &task_id,
            0,
            0,
            TaskStage::Failed,
            Some("PDF has no pages".into()),
        )
        .await;
        return Err(AppError::BadRequest("PDF has no pages".into()));
    }

    let mut sheets: Vec<ParsedSheet> = Vec::with_capacity(total);

    for (page_index, page_path) in page_paths.into_iter().enumerate() {
        emit(
            &progress_tx,
            &task_id,
            page_index,
            total,
            TaskStage::Rasterizing,
            None,
        )
        .await;

        let template_clone = template.clone();
        let layout_clone = layout;
        let page_path_clone = page_path.clone();

        let result = tokio::task::spawn_blocking(move || {
            process_one_page(
                &page_path_clone,
                &template_clone,
                &layout_clone,
                template_id,
            )
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("page join: {e}")))?;

        match result {
            Ok(sheet) => {
                emit(
                    &progress_tx,
                    &task_id,
                    page_index + 1,
                    total,
                    TaskStage::ReadingBubbles,
                    None,
                )
                .await;
                sheets.push(ParsedSheet {
                    page_index: page_index as u32,
                    ..sheet
                });
            }
            Err(e) => {
                emit(
                    &progress_tx,
                    &task_id,
                    page_index,
                    total,
                    TaskStage::Failed,
                    Some(format!("page {page_index}: {e}")),
                )
                .await;
                return Err(e);
            }
        }
    }

    emit(&progress_tx, &task_id, total, total, TaskStage::Done, None).await;
    Ok(sheets)
}

/// Single-page synchronous pipeline. Returns a [`ParsedSheet`] with `page_index`
/// left at `0`; the caller fills it in.
fn process_one_page(
    page_path: &Path,
    template: &OmrTemplate,
    layout: &MarkerLayout,
    template_id: i64,
) -> AppResult<ParsedSheet> {
    let raw = imgcodecs::imread(
        page_path
            .to_str()
            .ok_or_else(|| AppError::BadRequest("page path is not valid UTF-8".into()))?,
        imgcodecs::IMREAD_COLOR,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("imread: {e}")))?;
    if raw.empty() {
        return Err(AppError::BadRequest(format!(
            "could not decode raster {}",
            page_path.display()
        )));
    }

    // Stage: threshold (illumination flatten + adaptive Gaussian).
    let gray = threshold::to_gray(&raw)?;
    let pre_warp_binary = threshold::binarize(&gray)?;

    // Stage: deskew (Hough). Operates on binary; rotation is applied to gray.
    let (gray_level, _applied) = deskew::deskew(&gray, &pre_warp_binary)?;

    // Stage: detect markers in the (now level) gray image. ArUco prefers the
    // grayscale source over a thresholded mask — the detector runs its own thresholding.
    let markers = perspective::detect_corner_markers(&gray_level)?;

    // Stage: warp to canonical canvas. Feed the bg-subtracted "ink density" image
    // to the bubble sampler — adaptive thresholding under-marks uniformly filled
    // discs (see `threshold::binarize` doc) so we use the continuous signal.
    let warped = perspective::warp_to_canonical(&gray_level, &markers, layout)?;
    let warped_ink = threshold::flatten_to_ink(&warped)?;
    let readings = bubbles::read_bubbles(&warped_ink, template)?;

    Ok(ParsedSheet {
        template_id,
        page_index: 0,
        student_id_text: None, // P3-04 reads bubbles only; ID-text resolution lands in P3-05+.
        readings,
    })
}

/// Hash the template title into a stable `i64`. The persistence layer assigns the
/// real DB id; this is a placeholder that keeps the pipeline self-contained for
/// tests and for callers that have not yet stored the template.
fn derive_template_id(template: &OmrTemplate) -> i64 {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(template.title.as_bytes());
    h.update(template.version.to_le_bytes());
    let bytes = h.finalize();
    // Take the first 8 bytes as a signed i64 — collisions are tolerable here.
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    i64::from_le_bytes(buf)
}

async fn emit(
    tx: &tokio::sync::mpsc::Sender<TaskProgress>,
    task_id: &str,
    processed: usize,
    total: usize,
    stage: TaskStage,
    message: Option<String>,
) {
    let payload = TaskProgress {
        task_id: task_id.to_string(),
        processed: processed as u32,
        total: total as u32,
        stage,
        message,
    };
    // Best-effort emission; if the receiver is gone we keep going (the work has
    // already happened, dropping a progress event is harmless).
    let _ = tx.send(payload).await;
}
