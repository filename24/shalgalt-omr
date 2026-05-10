//! Grading result domain model.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub enum GradedAnswer {
    /// All required bubbles for this group were marked, no extras. Full score.
    Correct {
        group_id: String,
        marked_indices: Vec<u32>,
    },
    /// Marks present but they do not match the answer key. Zero score.
    Wrong {
        group_id: String,
        marked_indices: Vec<u32>,
        correct_indices: Vec<u32>,
    },
    /// No marks detected for the group. Zero score.
    Blank { group_id: String },
    /// More marks than the answer key allows for a single-correct question.
    /// Zero score (no partial credit when the question expects one answer).
    Multiple {
        group_id: String,
        marked_indices: Vec<u32>,
    },
    /// Multi-correct question with a strict subset of correct indices marked
    /// and no wrong extras. Score is `score_ratio * group_score`.
    Partial {
        group_id: String,
        marked_indices: Vec<u32>,
        correct_indices: Vec<u32>,
        /// `marked_correct / total_correct`, in `(0.0, 1.0)`.
        score_ratio: f32,
    },
    /// At least one bubble in the group sits in the `[0.35, 0.65]` uncertain
    /// band. The sheet is routed to manual review (P3-07) and scored zero
    /// until a human disambiguates the readings.
    Uncertain {
        group_id: String,
        uncertain_indices: Vec<u32>,
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
    /// Resolved `students.id` row, populated by the persistence layer after
    /// matching `student_id_text` against the roster.
    #[ts(type = "number | null")]
    pub student_id: Option<i64>,
    /// Raw student identifier read off the bubble sheet (e.g. `"00123"`).
    /// `None` when the template has no `BubbleKind::StudentId` group or the
    /// CV pipeline could not recover a value.
    #[ts(optional, type = "string")]
    pub student_id_text: Option<String>,
    pub total_score: f32,
    pub answers: Vec<GradedAnswer>,
    /// Absolute path to the result image, exposable via `asset://`.
    #[ts(optional, type = "string")]
    pub image_path: Option<String>,
    /// `true` when at least one [`GradedAnswer::Uncertain`] is present. The
    /// frontend mirrors this into `results.needs_review` and surfaces the
    /// sheet in the manual-review queue.
    pub needs_review: bool,
}
