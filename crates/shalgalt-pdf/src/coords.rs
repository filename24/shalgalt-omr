//! Conversion between normalized template coordinates and millimetre page coordinates.
//!
//! Master plan §9.2: template coordinates live in `[0, 1]`. P2-05 only needs the full-page
//! mapping below; P2-06 will add a margin-aware variant that body content (header, bubble
//! grid, sidebar) routes through.
//!
//! All conversions return PDF coordinates (origin at the bottom-left, y axis pointing up).
//! Wrap the result in `printpdf::Mm` to feed it directly to drawing ops.

use shalgalt_core::domain::PaperSpec;

/// Map `(tx, ty)` template coordinates (top-left origin, y pointing down) to PDF page
/// coordinates in millimetres (bottom-left origin, y pointing up).
///
/// Margins are ignored — corner markers and other elements that may bleed outside the
/// printable area use this mapping. Body content lands in P2-06's `project()` helper.
pub fn to_page_mm(tx: f64, ty: f64, paper: &PaperSpec) -> (f64, f64) {
    let x_mm = tx * paper.width_mm;
    // Template y grows downward, PDF y grows upward — flip.
    let y_mm = (1.0 - ty) * paper.height_mm;
    (x_mm, y_mm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_origin_to_top_left_in_pdf_coords() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = to_page_mm(0.0, 0.0, &paper);
        assert!((x - 0.0).abs() < 1e-9);
        assert!((y - paper.height_mm).abs() < 1e-9);
    }

    #[test]
    fn maps_one_one_to_bottom_right_in_pdf_coords() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = to_page_mm(1.0, 1.0, &paper);
        assert!((x - paper.width_mm).abs() < 1e-9);
        assert!(y.abs() < 1e-9);
    }

    #[test]
    fn maps_center_to_center() {
        let paper = PaperSpec::A4_PORTRAIT;
        let (x, y) = to_page_mm(0.5, 0.5, &paper);
        assert!((x - paper.width_mm / 2.0).abs() < 1e-9);
        assert!((y - paper.height_mm / 2.0).abs() < 1e-9);
    }
}
