//! 채점 오케스트레이터 — PDF 분해 → 페이지별 perspective transform → 버블 판독 → 결과 저장.
//!
//! P0 스텁: 진행률 이벤트 페이로드 타입만 정의한다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStage {
    LoadingPdf,
    Rasterizing,
    DetectingMarkers,
    ReadingBubbles,
    Grading,
    Saving,
    Done,
    Failed,
}

/// `task-progress` 이벤트의 페이로드. 프론트엔드 `progress.svelte.ts` 스토어가 구독한다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub task_id: String,
    pub processed: u32,
    pub total: u32,
    pub stage: TaskStage,
    /// 사람이 읽을 보조 메시지 (옵션).
    #[serde(default)]
    pub message: Option<String>,
}
