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
