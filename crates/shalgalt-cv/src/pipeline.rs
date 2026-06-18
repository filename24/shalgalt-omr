//! End-to-end OMR pipeline orchestrator.
//!
//! Wires the [`pdf`], [`threshold`], [`deskew`], [`perspective`], and [`bubbles`]
//! stages together and emits [`TaskProgress`] events at every stage transition.
//! Synchronous OpenCV / pdfium work runs inside a [`tokio::task::spawn_blocking`]
//! so the caller's runtime stays responsive (Rule 2).

use std::path::{Path, PathBuf};

use opencv::{imgcodecs, prelude::*};

use shalgalt_core::domain::{
    BubbleKind, BubbleReading, OmrTemplate, ParsedSheet, TaskProgress, TaskStage,
};
use shalgalt_core::error::{AppError, AppResult};

use crate::{
    bubbles, deskew, pdf,
    perspective::{self, MarkerLayout},
    threshold, AnswerKeyReading, GroupReading,
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

    // Do NOT emit `TaskStage::Done` here. This is only the CV half of the job —
    // the grading command (`apps/desktop/src/commands/scan.rs`) still has to score
    // each sheet, emit a `task-result` per page, and only then emit the terminal
    // `Done`. Emitting `Done` now races the frontend into finalizing the job before
    // any `task-result` arrives, persisting an empty `graded_sheets` array. The
    // orchestrating caller owns the terminal stage (see cv `AGENTS.md` Rule 2).
    Ok(sheets)
}

/// Read a single hand-filled OMR sheet as an answer key (P4-06).
///
/// This is the implementation behind the crate-level [`crate::read_answer_key`]
/// facade. It reuses the grading pipeline's per-page CV stages but enforces the
/// answer-key constraints: the source must be exactly one page, the template must
/// declare at least one question group, and a marker-detection failure is surfaced
/// as the dedicated [`AppError::AnswerKeyMarkerMissing`] rather than a generic
/// `BadRequest`. Returns one [`GroupReading`] per question group, in template order.
pub(crate) async fn run_answer_key(
    pdf_path: PathBuf,
    template: OmrTemplate,
    progress_tx: tokio::sync::mpsc::Sender<TaskProgress>,
    task_id: String,
    cache_dir: PathBuf,
) -> AppResult<AnswerKeyReading> {
    let layout = MarkerLayout::from_template(&template.markers);

    // A template with no question groups has nothing to read against — fail fast
    // before touching the (potentially large) PDF.
    let question_count = template
        .groups
        .iter()
        .filter(|g| g.kind == BubbleKind::Question)
        .count();
    if question_count == 0 {
        return Err(AppError::AnswerKeyTemplateMismatch(
            "template declares no question groups".into(),
        ));
    }

    emit(&progress_tx, &task_id, 0, 1, TaskStage::LoadingPdf, None).await;

    let raster_dir = cache_dir.join("page-rasters").join(&task_id);
    let pdf_path_for_blocking = pdf_path.clone();
    let raster_dir_clone = raster_dir.clone();
    let page_paths: Vec<PathBuf> = tokio::task::spawn_blocking(move || {
        pdf::rasterize_all_pages(&pdf_path_for_blocking, &raster_dir_clone)
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("rasterize join: {e}")))??;

    let page_count = page_paths.len() as u32;
    if page_count == 0 {
        return Err(AppError::BadRequest("PDF has no pages".into()));
    }
    // The answer sheet is a single page by contract (P4-06). Refuse multi-page
    // input rather than silently reading only the first page.
    if page_count > 1 {
        emit(
            &progress_tx,
            &task_id,
            0,
            1,
            TaskStage::Failed,
            Some("answer key sheet must be a single page".into()),
        )
        .await;
        return Err(AppError::AnswerKeyMultiPage);
    }

    // `read_answer_key_page` performs both marker detection and the bubble sweep in one
    // blocking call, so emit both stage labels up front rather than after the work.
    emit(
        &progress_tx,
        &task_id,
        0,
        1,
        TaskStage::DetectingMarkers,
        None,
    )
    .await;
    emit(
        &progress_tx,
        &task_id,
        0,
        1,
        TaskStage::ReadingBubbles,
        None,
    )
    .await;

    // The `page_count` guards above guarantee exactly one page; guard the `next()`
    // explicitly anyway (the repo forbids panics on runtime-derived values).
    let page_path = page_paths.into_iter().next().ok_or_else(|| {
        AppError::Internal(anyhow::anyhow!(
            "rasterizer returned no pages despite page_count == 1"
        ))
    })?;
    let template_clone = template.clone();
    let readings: Vec<BubbleReading> = tokio::task::spawn_blocking(move || {
        read_answer_key_page(&page_path, &template_clone, &layout)
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("page join: {e}")))??;

    // Collapse per-bubble readings into one ordered reading per question group.
    let groups: Vec<GroupReading> = template
        .groups
        .iter()
        .filter(|g| g.kind == BubbleKind::Question)
        .map(|g| {
            let fills: Vec<f32> = (0..g.bubbles.len() as u32)
                .map(|idx| {
                    readings
                        .iter()
                        .find(|r| r.group_id == g.id && r.bubble_index == idx)
                        .map(|r| r.fill)
                        .unwrap_or(0.0)
                })
                .collect();
            GroupReading {
                group_id: g.id.clone(),
                fills,
            }
        })
        .collect();

    emit(&progress_tx, &task_id, 1, 1, TaskStage::Done, None).await;

    Ok(AnswerKeyReading { groups, page_count })
}

/// Single-page synchronous answer-key reader. Mirrors [`process_one_page`]'s CV
/// stages but maps a marker-detection miss to the answer-key-specific error so the
/// UI can prompt the teacher to re-scan with all four markers visible.
fn read_answer_key_page(
    page_path: &Path,
    template: &OmrTemplate,
    layout: &MarkerLayout,
) -> AppResult<Vec<BubbleReading>> {
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

    let gray = threshold::to_gray(&raw)?;
    let pre_warp_binary = threshold::binarize(&gray)?;
    let (gray_level, _applied) = deskew::deskew(&gray, &pre_warp_binary)?;

    let markers = perspective::detect_corner_markers(&gray_level)
        .map_err(|e| AppError::AnswerKeyMarkerMissing(e.to_string()))?;

    let warped = perspective::warp_to_canonical(&gray_level, &markers, layout)?;
    let warped_ink = threshold::flatten_to_ink(&warped)?;
    bubbles::read_bubbles(&warped_ink, template)
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

    // Decode the student cipher ("Шифр") from the StudentId rows. The decode is
    // pure logic and lives in core so it stays testable without OpenCV; it returns
    // `None` when the code is blank or ambiguous, leaving the frontend to fall back
    // to a generated label.
    let student_id_text = shalgalt_core::decode_student_id(template, &readings);

    Ok(ParsedSheet {
        template_id,
        page_index: 0,
        student_id_text,
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
