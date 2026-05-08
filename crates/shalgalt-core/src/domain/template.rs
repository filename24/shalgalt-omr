//! OMR template domain model — the serialization unit defined in Rule 3.
//!
//! All coordinates are expressed in the **template canvas's normalized space (0.0 – 1.0)**.
//! Pixel coordinates of the actual scanned image are derived during the
//! `scan::perspective::warp` step. This policy lets a single template be reused across PDFs
//! at different DPIs and page sizes.
//!
//! `#[derive(TS)]` emits the matching TypeScript interfaces into
//! `src/lib/types/generated/` (P2-04). Run `pnpm generate-types` after editing
//! these structs.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct TemplatePoint {
    pub x: f32,
    pub y: f32,
}

/// Visual style of the printed marker. The P2-06 PDF renderer paints `Square` as a solid
/// block; `Aruco6x6` falls back to the same solid square placeholder until P3-01 (CV
/// migration) lands and rasterizes the actual bit grid.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub enum MarkerKind {
    /// Empty solid square — the legacy 4-corner OMR marker.
    #[default]
    Square,
    /// 6×6 ArUco marker. `ids[i]` indexes `aruco_dict::DICT_6X6_50` in TL/TR/BR/BL order.
    /// The renderer will stamp the actual bit grid once P3-01 ships; until then it draws
    /// the same solid square as `Square`.
    Aruco6x6 { ids: [u8; 4] },
}

impl MarkerKind {
    /// Helper for `serde(skip_serializing_if = ...)`. When the value is the default
    /// (`Square`) we omit the field on the wire so existing fixtures and storage JSON keep
    /// byte-for-byte compatibility.
    fn is_default(&self) -> bool {
        matches!(self, MarkerKind::Square)
    }
}

/// Reference point used for the 4-corner perspective transform.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct Marker {
    pub id: String,
    pub position: TemplatePoint,
    /// Width / height of the printed marker, in normalized coordinates.
    pub size: f32,
    /// Visual marker style. Defaults to [`MarkerKind::Square`] for serialization
    /// compatibility — pre-P2-06 templates that omit the field still deserialize cleanly.
    #[serde(default, skip_serializing_if = "MarkerKind::is_default")]
    pub kind: MarkerKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub enum BubbleKind {
    /// Student-id input region.
    StudentId,
    /// Regular question.
    Question,
}

/// A group of bubbles that share semantics (e.g. "question 1", "tens digit of student id").
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
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
    #[ts(optional, type = "string")]
    pub section: Option<String>,
}

fn default_score() -> f32 {
    1.0
}

/// Top-level structure persisted verbatim into the `templates.json_schema` column.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
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
