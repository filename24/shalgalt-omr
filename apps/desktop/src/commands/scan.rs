//! Batch PDF grading IPC.
//!
//! Rule 1·2: arguments are **local absolute path strings** and the heavy work runs in a
//! `tokio::spawn` background task. The command returns immediately and reports progress
//! through emitted events only.
//!
//! Two events flow back to the frontend:
//!   * `task-progress` — incremental stage updates from the CV pipeline plus the grading
//!     loop. Shape: `shalgalt_core::domain::TaskProgress`.
//!   * `task-result` — one event per graded page, carrying both the raw `ParsedSheet`
//!     and the scored `GradedSheet` plus the absolute path to the page raster (so the
//!     frontend can show it via `asset://localhost/`). Shape: [`TaskResult`].
//!
//! DB rule: this module never writes to the database directly. The `/grade` page owns
//! job-row persistence through `tauri-plugin-sql`.

use std::path::{Path, PathBuf};

use serde::Serialize;
use shalgalt_core::domain::{AnswerKey, GradedSheet, OmrTemplate, ParsedSheet, TaskResult};
use shalgalt_core::error::{AppError, AppResult};
use shalgalt_core::grading;
use shalgalt_cv::{preview, TaskProgress, TaskStage};
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;

const PROGRESS_EVENT: &str = "task-progress";
const RESULT_EVENT: &str = "task-result";

/// Kick off a batch grade run. Returns immediately; progress and per-page
/// results stream back via `task-progress` and `task-result` events keyed by
/// `task_id`.
///
/// Inputs:
///   * `task_id` — frontend-generated UUID. Lets the frontend correlate events
///     with the DB row it just inserted.
///   * `pdf_path` — absolute path on disk (Rule 1).
///   * `template_json` — serialized `OmrTemplate` from `templates.json_schema`.
///     Forwarding the full JSON keeps this command DB-free.
///   * `answer_key_json` — serialized `AnswerKey` pasted by the user.
#[tauri::command]
pub async fn scan_grade_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
    pdf_path: String,
    template_json: String,
    answer_key_json: String,
) -> AppResult<()> {
    if task_id.trim().is_empty() {
        return Err(AppError::BadRequest("task_id is empty".into()));
    }
    if pdf_path.trim().is_empty() {
        return Err(AppError::BadRequest("pdf_path is empty".into()));
    }

    let template: OmrTemplate = serde_json::from_str(&template_json)
        .map_err(|e| AppError::BadRequest(format!("template_json parse failed: {e}")))?;
    let answer_key: AnswerKey = serde_json::from_str(&answer_key_json)
        .map_err(|e| AppError::BadRequest(format!("answer_key_json parse failed: {e}")))?;

    let cache_dir = state.dirs().cache_dir.clone();
    let app_handle = app.clone();
    let task_id_owned = task_id.clone();

    // Rule 2: heavy work happens off the IPC thread. The pipeline emits its own
    // progress events; we wrap any error into a single `failed` stage so the
    // frontend's progress store can release the spinner.
    tokio::spawn(async move {
        if let Err(err) = run_grade_pipeline(
            &app_handle,
            &task_id_owned,
            PathBuf::from(pdf_path),
            template,
            answer_key,
            cache_dir,
        )
        .await
        {
            let _ = app_handle.emit(
                PROGRESS_EVENT,
                TaskProgress {
                    task_id: task_id_owned.clone(),
                    processed: 0,
                    total: 0,
                    stage: TaskStage::Failed,
                    message: Some(err.to_string()),
                },
            );
        }
    });

    Ok(())
}

async fn run_grade_pipeline(
    app: &AppHandle,
    task_id: &str,
    pdf_path: PathBuf,
    template: OmrTemplate,
    answer_key: AnswerKey,
    cache_dir: PathBuf,
) -> AppResult<()> {
    // Forward every TaskProgress message the CV pipeline produces. We use a
    // bounded channel so a stuck UI cannot make the pipeline hold unbounded
    // memory, and a dedicated forwarder task keeps the pipeline decoupled
    // from Tauri's emit API.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<TaskProgress>(64);
    let app_for_forwarder = app.clone();
    let forwarder = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = app_for_forwarder.emit(PROGRESS_EVENT, msg);
        }
    });

    let sheets = shalgalt_cv::process_pdf(&pdf_path, &template, tx, task_id, &cache_dir).await?;
    // The CV pipeline closes its end of the channel when it returns, so the
    // forwarder will drain and exit on its own. Awaiting it here flushes any
    // final messages before we move on to grading.
    let _ = forwarder.await;

    let total = sheets.len() as u32;

    // Page rasters are written by `shalgalt_cv::pdf::rasterize_all_pages` into
    // `<cache_dir>/page-rasters/<task_id>/page-NNN.png`. Reconstruct the path
    // for each sheet so `/review` can `asset://`-load it later.
    let raster_dir = cache_dir.join("page-rasters").join(task_id);

    // Stage transition: grading. Pre-bump processed=0 so the bar resets.
    let _ = app.emit(
        PROGRESS_EVENT,
        TaskProgress {
            task_id: task_id.to_string(),
            processed: 0,
            total,
            stage: TaskStage::Grading,
            message: None,
        },
    );

    for (i, parsed) in sheets.iter().enumerate() {
        let graded = grading::grade(&template, parsed, &answer_key)?;
        let page_image_path = page_raster_path(&raster_dir, parsed.page_index);

        let _ = app.emit(
            RESULT_EVENT,
            TaskResult {
                task_id: task_id.to_string(),
                page_index: parsed.page_index,
                page_image_path,
                parsed: parsed.clone(),
                graded,
            },
        );

        let _ = app.emit(
            PROGRESS_EVENT,
            TaskProgress {
                task_id: task_id.to_string(),
                processed: (i + 1) as u32,
                total,
                stage: TaskStage::Grading,
                message: None,
            },
        );
    }

    let _ = app.emit(
        PROGRESS_EVENT,
        TaskProgress {
            task_id: task_id.to_string(),
            processed: total,
            total,
            stage: TaskStage::Done,
            message: None,
        },
    );

    Ok(())
}

/// Mirror of `shalgalt_cv::pdf::rasterize_all_pages` filename convention. The
/// CV crate writes `page-{idx:04}.png` (0-based, 4-digit zero-padded) into
/// `raster_dir`; we reconstruct the same string here so we never have to
/// re-rasterize.
fn page_raster_path(raster_dir: &Path, page_index: u32) -> String {
    raster_dir
        .join(format!("page-{:04}.png", page_index))
        .to_string_lossy()
        .into_owned()
}

/// Re-grade one sheet using the supplied (potentially overridden) parsed
/// readings. Used by `/review/[job_id]` after the user disambiguates a flagged
/// bubble. Pure call into `shalgalt_core::grading::grade`.
#[tauri::command]
pub async fn regrade_sheet(
    template_json: String,
    parsed_sheet_json: String,
    answer_key_json: String,
) -> AppResult<GradedSheet> {
    let template: OmrTemplate = serde_json::from_str(&template_json)
        .map_err(|e| AppError::BadRequest(format!("template_json parse failed: {e}")))?;
    let parsed: ParsedSheet = serde_json::from_str(&parsed_sheet_json)
        .map_err(|e| AppError::BadRequest(format!("parsed_sheet_json parse failed: {e}")))?;
    let answer_key: AnswerKey = serde_json::from_str(&answer_key_json)
        .map_err(|e| AppError::BadRequest(format!("answer_key_json parse failed: {e}")))?;

    grading::grade(&template, &parsed, &answer_key)
}

/// Result of reading a hand-filled answer sheet (P4-06). Serialized straight back to
/// the caller (not via an event) so the review screen can render before persisting.
/// `answers`, `uncertain_groups`, and `group_ids` are index-aligned: `answers[i]` is the
/// read option for the question group `group_ids[i]`, and a value of `-1` means blank.
#[derive(Debug, Serialize)]
pub struct AnswerKeyImportSummary {
    /// Most-filled option index per question group (template order); `-1` for blank.
    pub answers: Vec<i32>,
    /// Indices into `answers` whose group landed in the `[0.35, 0.65]` uncertain band.
    pub uncertain_groups: Vec<u32>,
    /// `BubbleGroup.id` per entry, aligned with `answers`, so the UI persists without
    /// re-deriving the question-group order.
    pub group_ids: Vec<String>,
    /// Page count of the source PDF — always `1` on success (multi-page is rejected).
    pub page_count: u32,
    /// Absolute path to the scanned PDF, echoed back (Rule 1 — path, never bytes).
    pub source_path: String,
    /// Absolute path to the rasterized page PNG, for the side-by-side review preview.
    /// Lives under `$APPCACHE`, so the frontend loads it via `asset://localhost/`.
    pub preview_path: String,
}

/// Read one hand-filled OMR sheet as the canonical answer key for `(exam_id, variant)`
/// (P4-06). Reuses the grading CV pipeline via [`shalgalt_cv::read_answer_key`].
///
/// Rule 2 note: unlike `scan_grade_pdf` this command awaits its result instead of
/// returning a `task_id`, because the caller needs the readings to render the review
/// screen. The UI does not freeze — all CPU-bound CV work runs inside `spawn_blocking`
/// within `read_answer_key`, and stage updates stream on the shared `task-progress`
/// channel while the command future is awaited on Tauri's async runtime.
///
/// `exam_id` / `variant` are not persisted here (the frontend owns DB writes); they are
/// logged for diagnostics and let the caller correlate the scan with its target row.
#[tauri::command]
pub async fn scan_answer_key(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
    pdf_path: String,
    template_json: String,
    exam_id: i64,
    variant: String,
) -> AppResult<AnswerKeyImportSummary> {
    if task_id.trim().is_empty() {
        return Err(AppError::BadRequest("task_id is empty".into()));
    }
    if pdf_path.trim().is_empty() {
        return Err(AppError::BadRequest("pdf_path is empty".into()));
    }
    if variant.trim().is_empty() {
        return Err(AppError::BadRequest("variant is empty".into()));
    }

    let template: OmrTemplate = serde_json::from_str(&template_json)
        .map_err(|e| AppError::BadRequest(format!("template_json parse failed: {e}")))?;

    let cache_dir = state.dirs().cache_dir.clone();
    let pdf = PathBuf::from(&pdf_path);

    tracing::info!(exam_id, variant = %variant, "scan_answer_key: reading {pdf_path}");

    // Forward CV progress on the same `task-progress` channel grading uses, via a
    // dedicated task so the pipeline stays decoupled from Tauri's emit API.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<TaskProgress>(64);
    let app_for_forwarder = app.clone();
    let forwarder = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = app_for_forwarder.emit(PROGRESS_EVENT, msg);
        }
    });

    let reading = shalgalt_cv::read_answer_key(&pdf, &template, tx, &task_id, &cache_dir).await?;
    // The pipeline drops its channel end on return; flush any trailing progress.
    let _ = forwarder.await;

    let answers: Vec<i32> = reading
        .groups
        .iter()
        .map(|g| g.most_filled_index().map(|i| i as i32).unwrap_or(-1))
        .collect();
    let uncertain_groups: Vec<u32> = reading
        .groups
        .iter()
        .enumerate()
        .filter(|(_, g)| g.is_uncertain())
        .map(|(i, _)| i as u32)
        .collect();
    let group_ids: Vec<String> = reading.groups.iter().map(|g| g.group_id.clone()).collect();

    // `read_answer_key` rasterized the single page into
    // `<cache_dir>/page-rasters/<task_id>/page-0000.png`. Reuse it as the review
    // preview rather than re-rasterizing.
    let raster_dir = cache_dir.join("page-rasters").join(&task_id);
    let preview_path = page_raster_path(&raster_dir, 0);

    Ok(AnswerKeyImportSummary {
        answers,
        uncertain_groups,
        group_ids,
        page_count: reading.page_count,
        source_path: pdf_path,
        preview_path,
    })
}

/// Rasterize the first page of a PDF into a PNG and return the absolute cache
/// path. Used by the P1 template editor to import a PDF as the canvas
/// backdrop. Heavy work runs on a blocking thread (pdfium-render is sync).
#[tauri::command]
pub async fn rasterize_pdf_first_page(
    state: State<'_, AppState>,
    pdf_path: String,
) -> AppResult<String> {
    if pdf_path.trim().is_empty() {
        return Err(AppError::BadRequest("pdf_path is empty".into()));
    }
    let pdf = PathBuf::from(&pdf_path);
    let cache_dir = state.dirs().cache_dir.clone();

    let dest = tokio::task::spawn_blocking(move || preview::rasterize_first_page(&pdf, &cache_dir))
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("join error: {e}")))??;

    Ok(dest.to_string_lossy().into_owned())
}
