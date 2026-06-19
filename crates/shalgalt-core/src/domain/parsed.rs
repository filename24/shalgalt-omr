//! Output of the CV pipeline — the input to the grading engine.
//!
//! The CV crate (`shalgalt-cv`) emits a [`ParsedSheet`] per page after marker
//! detection, perspective warp, and bubble fill-ratio measurement. The grading
//! engine ([`crate::grading`]) consumes these readings together with an
//! [`AnswerKey`](super::answer_key::AnswerKey) to produce
//! [`GradedSheet`](super::result::GradedSheet).
//!
//! `BubbleReading::fill` is the dark-pixel ratio in `[0.0, 1.0]`. The decision
//! bands (`<0.35` unfilled, `>0.65` filled, `[0.35, 0.65]` uncertain) are locked
//! by ADR `0006-confidence-band` (P3) and exposed as constants on
//! [`BubbleReading`] so consumers do not hand-roll the thresholds.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One bubble's measurement after CV processing.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct BubbleReading {
    /// Matches [`BubbleGroup::id`](super::template::BubbleGroup::id).
    pub group_id: String,
    /// Index into [`BubbleGroup::bubbles`](super::template::BubbleGroup::bubbles).
    pub bubble_index: u32,
    /// Dark-pixel ratio in `[0.0, 1.0]`. See [`BubbleReading::FILL_FILLED_MIN`]
    /// and [`BubbleReading::FILL_UNFILLED_MAX`] for the decision bands.
    pub fill: f32,
    /// CV pipeline's confidence in the fill measurement, `[0.0, 1.0]`. Higher
    /// is better. The grading engine does not use this directly today; it is
    /// surfaced to the manual-review UI (P3-07) for sorting.
    pub confidence: f32,
}

impl BubbleReading {
    /// `fill < FILL_UNFILLED_MAX` ⇒ unfilled.
    pub const FILL_UNFILLED_MAX: f32 = 0.35;
    /// `fill > FILL_FILLED_MIN` ⇒ filled.
    pub const FILL_FILLED_MIN: f32 = 0.65;

    /// True when the bubble is confidently filled (`fill > 0.65`).
    pub fn is_filled(&self) -> bool {
        self.fill > Self::FILL_FILLED_MIN
    }

    /// True when the bubble is confidently empty (`fill < 0.35`).
    pub fn is_unfilled(&self) -> bool {
        self.fill < Self::FILL_UNFILLED_MAX
    }

    /// True when the fill ratio sits in the `[0.35, 0.65]` uncertain band.
    /// A sheet with any uncertain bubble is flagged `needs_review`.
    pub fn is_uncertain(&self) -> bool {
        !self.is_filled() && !self.is_unfilled()
    }
}

/// CV output for a single page of a scanned exam.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct ParsedSheet {
    #[ts(type = "number")]
    pub template_id: i64,
    /// 0-based page index inside the source PDF.
    pub page_index: u32,
    /// Student identifier read from `BubbleKind::StudentId` groups, when
    /// present. The grading engine passes this through to
    /// [`GradedSheet::student_id_text`](super::result::GradedSheet::student_id_text);
    /// resolution to the `students` table row id is the persistence layer's job.
    #[ts(optional, type = "string")]
    pub student_id_text: Option<String>,
    /// Exam-form variant decoded from the `BubbleKind::Variant` row, as the
    /// printed letter (`"A"`, `"B"`, …). `None` when the template has no variant
    /// row, or the mark is blank/ambiguous. The grading orchestrator uses this to
    /// pick the matching answer key for a mixed-variant batch; an unresolved
    /// variant flags the sheet for manual review.
    #[ts(optional, type = "string")]
    pub variant: Option<String>,
    pub readings: Vec<BubbleReading>,
}
