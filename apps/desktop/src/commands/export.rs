//! Result xlsx export IPC (P5-01).
//!
//! The frontend gathers graded sheets out of SQLite (`jobs.graded_sheets_json`,
//! via plugin-sql), shapes them into an [`XlsxReport`] plus a [`ReportLabels`]
//! bundle (Mongolian strings from the P1 table), and calls this command with an
//! `output_path` chosen through the save dialog.
//!
//! Rule 1: only path strings and small JSON cross the boundary — the workbook
//! bytes are produced and written Rust-side, never shipped back to the webview.
//! The heavy work (zip compression + file write) runs on the blocking pool so the
//! async runtime is never stalled (Rule 2 spirit, without the batch/progress
//! contract — export is a single request/response).
//!
//! Payload size note: `report` carries the graded answers as JSON, which the
//! webview already holds in memory from `jobs.graded_sheets_json`. For the
//! target use case (a teacher grading one class — tens of sheets, ~50 questions)
//! this is well under ~100 KB, consistent with the per-sheet `TaskResult` events
//! grading already streams across IPC. If large-cohort exports become a
//! requirement, switch to writing `report` to a temp JSON file and passing only
//! its path (the Rule 1 path-string pattern).

use shalgalt_core::error::{AppError, AppResult};
use shalgalt_core::export::{build_report, ReportLabels, XlsxReport};

/// Render `report` to xlsx and write it to `output_path`, returning that path.
#[tauri::command]
pub async fn export_results_xlsx(
    report: XlsxReport,
    labels: ReportLabels,
    output_path: String,
) -> AppResult<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = build_report(&report, &labels)?;
        std::fs::write(&output_path, &bytes).map_err(|e| {
            AppError::Internal(anyhow::anyhow!(
                "failed to write xlsx to {output_path:?}: {e}"
            ))
        })?;
        Ok(output_path)
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("xlsx export task panicked: {e}")))?
}
