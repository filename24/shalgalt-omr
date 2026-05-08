//! Conversion between normalized template coordinates and millimetre page coordinates.
//!
//! Master plan §9.2: template coordinates live in `[0, 1]`. This module exposes two
//! mappings:
//!
//! - [`to_page_mm`] — full-page mapping that ignores the margin. Used by anything that
//!   may bleed outside the printable area (corner markers, page background).
//! - [`project`] — margin-aware mapping into the **printable region**. Body content
//!   (header, bubbles, sidebar) routes through this single entry point.
//!
//! Both helpers return PDF coordinates (origin at the bottom-left, y axis pointing up).
//! Wrap the result in `printpdf::Mm` to feed it directly to drawing ops.

use printpdf::Mm;
use shalgalt_core::domain::PaperSpec;

/// Map `(tx, ty)` template coordinates (top-left origin, y pointing down) to PDF page
/// coordinates in millimetres (bottom-left origin, y pointing up).
///
/// Margins are ignored. Corner markers and page-background elements use this mapping.
pub fn to_page_mm(tx: f64, ty: f64, paper: &PaperSpec) -> (f64, f64) {
    let x_mm = tx * paper.width_mm;
    // Template y grows downward, PDF y grows upward — flip.
    let y_mm = (1.0 - ty) * paper.height_mm;
    (x_mm, y_mm)
}

/// Map `(tx, ty)` template coordinates onto the margin-aware printable rectangle.
///
/// This is the single entry point for body content (header, bubbles, sidebar).
pub fn project(tx: f64, ty: f64, paper: &PaperSpec) -> (Mm, Mm) {
    let printable_w = paper.width_mm - 2.0 * paper.margin_mm;
    let printable_h = paper.height_mm - 2.0 * paper.margin_mm;
    let x_mm = paper.margin_mm + tx * printable_w;
    // Flip y, then add the bottom margin.
    let y_mm = paper.margin_mm + (1.0 - ty) * printable_h;
    (Mm(x_mm as f32), Mm(y_mm as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_page_mm_origin_is_top_left() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = to_page_mm(0.0, 0.0, &paper);
        assert!((x - 0.0).abs() < 1e-9);
        assert!((y - paper.height_mm).abs() < 1e-9);
    }

    #[test]
    fn to_page_mm_one_one_is_bottom_right() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = to_page_mm(1.0, 1.0, &paper);
        assert!((x - paper.width_mm).abs() < 1e-9);
        assert!(y.abs() < 1e-9);
    }

    #[test]
    fn to_page_mm_center() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = to_page_mm(0.5, 0.5, &paper);
        assert!((x - paper.width_mm / 2.0).abs() < 1e-9);
        assert!((y - paper.height_mm / 2.0).abs() < 1e-9);
    }

    #[test]
    fn project_origin_is_top_left_inside_margin() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = project(0.0, 0.0, &paper);
        assert!((x.0 as f64 - paper.margin_mm).abs() < 1e-6);
        assert!((y.0 as f64 - (paper.height_mm - paper.margin_mm)).abs() < 1e-6);
    }

    #[test]
    fn project_one_one_is_bottom_right_inside_margin() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = project(1.0, 1.0, &paper);
        assert!((x.0 as f64 - (paper.width_mm - paper.margin_mm)).abs() < 1e-6);
        assert!((y.0 as f64 - paper.margin_mm).abs() < 1e-6);
    }
}
