//! Paper geometry shared between PDF generation and the editor preview.
//!
//! Lives in `shalgalt-core` so both `shalgalt-pdf` (printpdf-backed renderer) and the
//! frontend (via the `ts-rs` codegen from P2-04) consume a single source of truth.
//!
//! All dimensions are in **millimetres**. Template coordinates remain normalized to
//! `[0, 1]`; conversion to mm is centralized in `shalgalt-pdf::coords` (P2-05).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Physical dimensions and margins of a printed sheet. All units are millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct PaperSpec {
    /// Page width in millimetres.
    pub width_mm: f64,
    /// Page height in millimetres.
    pub height_mm: f64,
    /// Margin (mm) applied uniformly to all four sides of the printable area.
    pub margin_mm: f64,
    /// Page orientation. The visual meaning is determined by `width_mm`/`height_mm`
    /// themselves; this field carries the original intent as metadata.
    pub orientation: Orientation,
}

/// Print orientation. `width_mm < height_mm` is typically `Portrait`, but the field
/// is kept separate so the original intent is preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub enum Orientation {
    Portrait,
    Landscape,
}

impl PaperSpec {
    /// A4 portrait (210 × 297 mm), 12 mm margin. v1.0 default per master plan §6.2 / §9.2.
    pub const A4_PORTRAIT: PaperSpec = PaperSpec {
        width_mm: 210.0,
        height_mm: 297.0,
        margin_mm: 12.0,
        orientation: Orientation::Portrait,
    };
}

impl Default for PaperSpec {
    fn default() -> Self {
        Self::A4_PORTRAIT
    }
}
