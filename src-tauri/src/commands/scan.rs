//! Batch PDF grading IPC.
//!
//! Rule 1·2: arguments are **local absolute path strings** and the heavy work runs in a
//! `tokio::spawn` background task. The command returns a `task_id` immediately and reports
//! progress through emitted events only.
//!
//! DB rule: this command never writes to the database directly. Grading results are
//! announced via a `task-result` event; the frontend persists them through
//! `tauri-plugin-sql`.

use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::error::AppResult;
use crate::scan::{TaskProgress, TaskStage};
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
        return Err(crate::error::AppError::BadRequest(
            "pdf_path is empty".into(),
        ));
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
