//! `task-progress` IPC payload (Rule 2).
//!
//! The CV pipeline (`shalgalt_cv`) emits `TaskProgress` events through a
//! `tokio::sync::mpsc::Sender`; the desktop shell forwards them to the frontend
//! via `app.emit("task-progress", ...)`. The shape is part of the IPC contract,
//! so it lives next to the other domain models and ts-rs generates the matching
//! TypeScript interface in `src/lib/types/generated/`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::parsed::ParsedSheet;
use super::result::GradedSheet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct TaskProgress {
    pub task_id: String,
    pub processed: u32,
    pub total: u32,
    pub stage: TaskStage,
    /// Optional human-readable secondary message.
    #[serde(default)]
    pub message: Option<String>,
}

/// Per-page payload emitted on the `task-result` event after grading
/// succeeds (P3-06). Carries everything `/grade` and `/review` need to render
/// the sheet without a second IPC round-trip — including the absolute path to
/// the page raster so it can be loaded via `asset://localhost/` (Rule 1).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct TaskResult {
    pub task_id: String,
    pub page_index: u32,
    /// Absolute filesystem path to the rasterized page image.
    pub page_image_path: String,
    pub parsed: ParsedSheet,
    pub graded: GradedSheet,
}
