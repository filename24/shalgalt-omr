//! Batch PDF grading IPC.
//!
//! Rule 1·2: arguments are **local absolute path strings** and the heavy work runs in a
//! `tokio::spawn` background task. The command returns a `task_id` immediately and reports
//! progress through emitted events only.
//!
//! DB rule: this command never writes to the database directly. Grading results are
//! announced via a `task-result` event; the frontend persists them through
//! `tauri-plugin-sql`.

use std::path::PathBuf;

use shalgalt_core::error::{AppError, AppResult};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::scan::{preview, TaskProgress, TaskStage};
use crate::state::AppState;

const PROGRESS_EVENT: &str = "task-progress";

#[derive(serde::Serialize)]
pub struct ScanJob {
    pub task_id: String,
}

#[tauri::command]
pub async fn scan_grade_pdf(
    app: AppHandle,
    _state: State<'_, AppState>,
    pdf_path: String,
    template_id: i64,
) -> AppResult<ScanJob> {
    if pdf_path.trim().is_empty() {
        return Err(AppError::BadRequest("pdf_path is empty".into()));
    }

    let task_id = Uuid::new_v4().to_string();
    let task_id_for_task = task_id.clone();

    // P0: actual CV processing arrives in P2. For now, emit a single 0/0 progress event so
    // the channel can be smoke-tested end-to-end.
    tokio::spawn(async move {
        let _ = app.emit(
            PROGRESS_EVENT,
            TaskProgress {
                task_id: task_id_for_task.clone(),
                processed: 0,
                total: 0,
                stage: TaskStage::LoadingPdf,
                message: Some(format!("queued: {pdf_path} (template {template_id})")),
            },
        );
    });

    Ok(ScanJob { task_id })
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
