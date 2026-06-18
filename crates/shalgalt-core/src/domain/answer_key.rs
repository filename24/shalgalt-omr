//! Per-exam answer key — separates "what is correct" from "what was authored
//! into the template". One [`OmrTemplate`](super::template::OmrTemplate) can be
//! reused across many exams; each exam owns one or more variants (`A`/`B`/…),
//! and each variant carries its own [`AnswerKey`].
//!
//! Persisted in the `answer_keys.answers_json` column (see master plan §9.1
//! migration `0004_exams_and_answer_keys.sql`, lands in P4-04 — note that
//! migration `0003_jobs.sql` ships first as part of P3-08).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Correct bubble indices for one [`BubbleGroup`](super::template::BubbleGroup).
/// `correct_indices` is a list to support multi-correct questions (e.g. "B and
/// D"). Single-correct questions hold exactly one element.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct AnswerKeyEntry {
    pub group_id: String,
    pub correct_indices: Vec<u32>,
    /// Points this question is worth *in this exam*. `None` falls back to the
    /// template's [`BubbleGroup::score`](super::template::BubbleGroup::score), so
    /// one template can back exams that weight the same question differently and
    /// pre-existing keys (authored before per-exam scoring) keep their behavior.
    #[ts(optional, type = "number")]
    pub score: Option<f32>,
}

/// Answer key for one variant of one exam.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct AnswerKey {
    #[ts(type = "number")]
    pub exam_id: i64,
    /// Variant label, e.g. `"A"` / `"B"` / `"C"` / `"D"`. Must be unique within
    /// the exam (DB-level `UNIQUE (exam_id, variant)` enforces this).
    pub variant: String,
    pub answers: Vec<AnswerKeyEntry>,
}

impl AnswerKey {
    /// Look up the correct indices for a `BubbleGroup.id`, or `None` when the
    /// key has no entry for that group. The grading engine treats a missing
    /// entry as "this question is not part of the exam" and skips the group, so
    /// one physical template can back exams with fewer questions.
    pub fn correct_for(&self, group_id: &str) -> Option<&[u32]> {
        self.answers
            .iter()
            .find(|e| e.group_id == group_id)
            .map(|e| e.correct_indices.as_slice())
    }

    /// The per-exam points for a `BubbleGroup.id`, when this key overrides the
    /// template default. `None` means "use the template's `BubbleGroup.score`".
    pub fn score_for(&self, group_id: &str) -> Option<f32> {
        self.answers
            .iter()
            .find(|e| e.group_id == group_id)
            .and_then(|e| e.score)
    }
}
