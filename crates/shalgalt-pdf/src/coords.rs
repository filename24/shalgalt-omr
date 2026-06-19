//! Conversion between normalized template coordinates and millimetre page coordinates.
//!
//! Master plan §9.2: template coordinates live in `[0, 1]` and map directly to the
//! **full page**, not to a margin-inset rectangle. Both the editor canvas
//! (`fitContainOrA4` on the full A4 backdrop) and the CV pipeline (which warps the
//! ArUco-marked rectangle to the unit square) treat `[0, 1]` as the entire sheet —
//! this crate must do the same so a PDF rendered here, then printed and scanned (or
//! re-imported as a backdrop), keeps every bubble at the same normalized coordinate
//! it had in the template.
//!
//! The two helpers below therefore produce identical positions; only the return type
//! differs to satisfy printpdf's `Mm` ops:
//!
//! - [`to_page_mm`] returns bare `(f64, f64)` — used where caller-side arithmetic
//!   (e.g. marker corner offsets) is more convenient on raw mm.
//! - [`project`] wraps the same numbers in [`printpdf::Mm`] for direct hand-off to
//!   `printpdf` drawing operators.
//!
//! `PaperSpec::margin_mm` is **not** consumed here — it remains a hint used by
//! decorative chrome (header banner, sidebar wrap width, manual-entry underline
//! clamp) but never repositions template-bearing geometry. If you need a margin-
//! inset region for any new chrome, derive it locally rather than reintroducing a
//! second projection.

use printpdf::Mm;
use shalgalt_core::domain::PaperSpec;

/// Map `(tx, ty)` template coordinates (top-left origin, y pointing down) to PDF page
/// coordinates in millimetres (bottom-left origin, y pointing up).
///
/// Full-page mapping. `tx = 0.0` is the left edge, `tx = 1.0` is the right edge.
pub fn to_page_mm(tx: f64, ty: f64, paper: &PaperSpec) -> (f64, f64) {
    let x_mm = tx * paper.width_mm;
    // Template y grows downward, PDF y grows upward — flip.
    let y_mm = (1.0 - ty) * paper.height_mm;
    (x_mm, y_mm)
}

/// Same mapping as [`to_page_mm`] but wrapped in [`printpdf::Mm`] so the result feeds
/// drawing ops directly. Keep this as the single body-content entry point — it stays
/// in lockstep with the editor's full-page coordinate space (master plan §9.2).
pub fn project(tx: f64, ty: f64, paper: &PaperSpec) -> (Mm, Mm) {
    let (x_mm, y_mm) = to_page_mm(tx, ty, paper);
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

    /// Regression for the P3-06 follow-up: `project` MUST agree with `to_page_mm`
    /// on every input. Any divergence reintroduces the editor / PDF roundtrip
    /// drift that mis-aligned bubbles by ~10 mm at the top-left of A4.
    #[test]
    fn project_matches_to_page_mm() {
        let paper = PaperSpec::A4_PORTRAIT;
        for &(tx, ty) in &[
            (0.0, 0.0),
            (1.0, 1.0),
            (0.5, 0.5),
            (0.07, 0.27),
            (0.55, 0.82),
        ] {
            let (px, py) = project(tx, ty, &paper);
            let (mx, my) = to_page_mm(tx, ty, &paper);
            assert!(
                (px.0 as f64 - mx).abs() < 1e-3,
                "x mismatch at ({tx}, {ty})"
            );
            assert!(
                (py.0 as f64 - my).abs() < 1e-3,
                "y mismatch at ({tx}, {ty})"
            );
        }
    }

    #[test]
    fn project_origin_is_top_left_corner() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = project(0.0, 0.0, &paper);
        assert!((x.0 as f64).abs() < 1e-3);
        assert!((y.0 as f64 - paper.height_mm).abs() < 1e-3);
    }

    #[test]
    fn project_one_one_is_bottom_right_corner() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = project(1.0, 1.0, &paper);
        assert!((x.0 as f64 - paper.width_mm).abs() < 1e-3);
        assert!((y.0 as f64).abs() < 1e-3);
    }
}
