//! PDF 일괄 채점 IPC.
//!
//! Rule 1·2 적용: 인자는 **로컬 절대 경로 문자열**, 본 작업은 `tokio::spawn` 백그라운드.
//! 즉시 `task_id`만 반환하고, 실제 진행은 emit 이벤트로만 통보한다.
//!
//! Rule (DB): 채점 결과는 본 명령이 직접 DB에 쓰지 않고, `task-result` 이벤트로 발행한다.
//! 프론트엔드는 `tauri-plugin-sql`을 통해 결과 행을 INSERT 한다.

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

    // P0: 실제 CV 처리는 P2에서 채워진다. 일단 진행률 0/0 이벤트만 한 번 쏴서 채널 동작을 검증.
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
