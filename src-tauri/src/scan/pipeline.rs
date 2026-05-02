//! Grading orchestrator — PDF split → per-page perspective transform → bubble read → save.
//!
//! P0 stub: only the progress payload types are defined here.

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

/// Payload of the `task-progress` event consumed by the frontend `progress.svelte.ts`
/// store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub task_id: String,
    pub processed: u32,
    pub total: u32,
    pub stage: TaskStage,
    /// Optional human-readable secondary message.
    #[serde(default)]
    pub message: Option<String>,
}
