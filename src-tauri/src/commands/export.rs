//! 결과 엑셀 내보내기 IPC.
//!
//! P4: 프론트엔드가 plugin-sql로 모은 결과 배열을 인자로 받아 xlsx로 변환 후
//! 저장된 파일의 절대경로를 반환한다.

use tauri::State;

use crate::error::{AppError, AppResult};
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
