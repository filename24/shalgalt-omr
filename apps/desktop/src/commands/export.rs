//! Result xlsx export IPC.
//!
//! P4: receives the result array gathered by the frontend (via plugin-sql) and writes the
//! xlsx file, returning the absolute path to the file.

use shalgalt_core::error::{AppError, AppResult};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn export_results_xlsx(
    state: State<'_, AppState>,
    template_id: i64,
) -> AppResult<String> {
    let _ = state;
    let _ = template_id;
    Err(AppError::Internal(anyhow::anyhow!(
        "export_results_xlsx is not implemented yet (P4)"
    )))
}
