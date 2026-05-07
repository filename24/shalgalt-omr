//! OMR template domain model — the serialization unit defined in Rule 3.
//!
//! All coordinates are expressed in the **template canvas's normalized space (0.0 – 1.0)**.
//! Pixel coordinates of the actual scanned image are derived during the
//! `scan::perspective::warp` step. This policy lets a single template be reused across PDFs
//! at different DPIs and page sizes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemplatePoint {
    pub x: f32,
    pub y: f32,
}

/// Reference point used for the 4-corner perspective transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    pub id: String,
    pub position: TemplatePoint,
    /// Width / height of the printed marker, in normalized coordinates.
    pub size: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BubbleKind {
    /// Student-id input region.
    StudentId,
    /// Regular question.
    Question,
}

/// A group of bubbles that share semantics (e.g. "question 1", "tens digit of student id").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleGroup {
    pub id: String,
    pub kind: BubbleKind,
    /// Human-readable label ("Q1", "hundreds digit", ...).
    pub label: String,
    /// Coordinates of each option inside the group, in normalized space. Index 0 is usually
    /// the first label such as "1" or "0".
    pub bubbles: Vec<TemplatePoint>,
    /// Index of the correct answer (only meaningful for `Question`). `None` when unset.
    pub answer_index: Option<u32>,
    /// Score weight. Defaults to 1.0.
    #[serde(default = "default_score")]
    pub score: f32,
    /// Optional UI-only grouping label (e.g. "Шифр", "1-Р ХЭСЭГ", "2.1"). Used by the
    /// editor's LayerTree and the result table, ignored by the CV pipeline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

fn default_score() -> f32 {
    1.0
}

/// Top-level structure persisted verbatim into the `templates.json_schema` column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmrTemplate {
    /// Schema version — bump when the format breaks compatibility.
    #[serde(default = "current_version")]
    pub version: u32,
    pub title: String,
    /// The four markers in TL, TR, BR, BL order.
    pub markers: [Marker; 4],
    pub groups: Vec<BubbleGroup>,
}

fn current_version() -> u32 {
    1
}

impl OmrTemplate {
    pub const CURRENT_VERSION: u32 = 1;
}
