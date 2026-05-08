//! Rendering visual style — externally injected option bundles.
//!
//! All units are explicit (mm / Pt). Normalized coordinates never leak in here —
//! normalization is handled centrally by [`crate::coords`].

/// Visual parameters for a bubble (circle). Values must match what the CV pipeline
/// assumes in P3-01; see ADR `0002-pdf-generator-printpdf.md` for the locked contract.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BubbleStyle {
    /// Bubble (circle) diameter in millimetres. Default 2.5 mm.
    pub diameter_mm: f64,
    /// Outline thickness in millimetres. Default 0.4 mm.
    pub stroke_mm: f64,
    /// Horizontal offset (mm) of the bubble's printed label. Positive → to the right.
    pub label_offset_mm: f64,
    /// Label font size in points. Default 8 pt.
    pub label_size_pt: f64,
}

impl Default for BubbleStyle {
    fn default() -> Self {
        Self {
            diameter_mm: 2.5,
            stroke_mm: 0.4,
            label_offset_mm: 1.5,
            label_size_pt: 8.0,
        }
    }
}

/// Pre-translated text bundle for the page header. The renderer never imports an i18n
/// table; the caller (the P2-07 Tauri command) selects the appropriate Mongolian copy
/// and supplies it via this struct.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderText {
    /// Large title — typically equal to `template.title`, but the caller may override.
    pub title: Option<String>,
    /// Subtitle (e.g. grade / department). Omitted when `None`.
    pub subtitle: Option<String>,
    /// School name. Omitted when `None`.
    pub school: Option<String>,
    /// Teacher's name. Omitted when `None`.
    pub teacher: Option<String>,
}
