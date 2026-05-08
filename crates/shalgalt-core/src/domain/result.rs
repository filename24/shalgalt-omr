//! Grading result domain model.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub enum GradedAnswer {
    /// Single mark — correct answer.
    Correct { group_id: String, marked_index: u32 },
    /// Single mark — wrong answer.
    Wrong {
        group_id: String,
        marked_index: u32,
        correct_index: u32,
    },
    /// No mark detected.
    Blank { group_id: String },
    /// Multiple marks detected.
    Multiple {
        group_id: String,
        marked_indices: Vec<u32>,
    },
}

/// Grading result for a single OMR sheet — serialized into `results.detail_answers`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct GradedSheet {
    /// SQLite `INTEGER` IDs are emitted as `number` in TS so the frontend can
    /// keep using `number`-keyed objects (see `commands::results::ResultSummary`).
    #[ts(type = "number")]
    pub template_id: i64,
    #[ts(type = "number | null")]
    pub student_id: Option<i64>,
    pub total_score: f32,
    pub answers: Vec<GradedAnswer>,
    /// Absolute path to the result image, exposable via `asset://`.
    pub image_path: Option<String>,
}
