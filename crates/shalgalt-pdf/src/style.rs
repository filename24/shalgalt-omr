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
    /// Horizontal offset (mm) used by per-bubble label helpers. Kept on `BubbleStyle` so
    /// future label primitives (e.g. answer-key overlay in P5) share the same constant.
    pub label_offset_mm: f64,
    /// Label font size in points. Default 8 pt.
    pub label_size_pt: f64,
    /// Horizontal offset (mm) of a row label drawn to the LEFT of the first bubble in a
    /// group. Positive values move the label further left from the bubble centre.
    pub row_label_offset_mm: f64,
}

impl Default for BubbleStyle {
    fn default() -> Self {
        Self {
            // 3.0 mm matches the industry-standard OMR bubble diameter (≈ 40–50 px at
            // 300 DPI). Combined with a 1:1 edge-to-edge gap (pitch ≈ 6 mm), adjacent
            // bubbles stay visually separable to OpenCV's adaptive threshold pass and
            // ink bleed from a pen stroke does not connect into a single contour blob.
            diameter_mm: 3.0,
            stroke_mm: 0.3,
            label_offset_mm: 1.5,
            // Question / row label font is set to 10 pt (industry minimum legibility for
            // unaided reading at A4 distances).
            label_size_pt: 10.0,
            // Tight gap between handwriting underline and the cipher row — the row label
            // for cipher rows is empty by spec, so the offset only buffers the underline.
            row_label_offset_mm: 2.5,
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
