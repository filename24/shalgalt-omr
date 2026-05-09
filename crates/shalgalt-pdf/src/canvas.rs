//! Thin wrapper over `printpdf`'s low-level `Op` enum.
//!
//! Centralises the four bug-prone primitives — text positioning, BT/ET wrapping, stroke /
//! fill colour, line thickness — so call sites describe intent ("draw the row label here,
//! right-aligned, 7 pt") instead of emitting raw PDF ops. The cursor-accumulation bug
//! fixed in the P2-06 follow-up only had to be diagnosed once: every [`Canvas::text`]
//! call wraps its own `StartTextSection` / `EndTextSection`, so a second positioned write
//! cannot drift into a `Td` relative move accidentally.
//!
//! This module owns no business logic. It is purely a vocabulary layer between the layout
//! modules and `printpdf`. Anything stateful (font selection, label-array selection)
//! still lives in the caller.

use printpdf::{
    Color, FontId, Line, LinePoint, Mm, Op, PaintMode, PdfFontHandle, Point, Pt, Rgb, TextItem,
};

use crate::shapes;

/// Approximate `pt → mm` correction. 1 pt = 1/72 inch; 1 inch = 25.4 mm.
const PT_TO_MM: f64 = 25.4 / 72.0;

/// Average glyph advance / font-size ratio used by [`Canvas::text_right_aligned`].
/// Calibrated against NotoSans Latin + Cyrillic at 7–12 pt.
const AVG_GLYPH_ADVANCE_RATIO: f64 = 0.55;

/// Output buffer for one PDF page, exposed via intent-shaped helpers.
pub struct Canvas {
    ops: Vec<Op>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    /// Consume the canvas and return its accumulated ops in emit order.
    pub fn into_ops(self) -> Vec<Op> {
        self.ops
    }

    /// Append a raw `Op`. Escape hatch for layout modules that still need to drive
    /// printpdf directly (graphics-state save/restore, text matrix rotation, …).
    pub fn push(&mut self, op: Op) {
        self.ops.push(op);
    }

    /// Borrow the underlying op buffer. Used by tests that count `Op::ShowText` etc.
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    /// Set the current outline colour to black and the outline thickness to
    /// `thickness_mm` millimetres. State is sticky in PDF — call once per run of
    /// homogenous strokes.
    pub fn set_stroke_black(&mut self, thickness_mm: f64) {
        self.ops.push(Op::SetOutlineColor { col: black() });
        self.ops.push(Op::SetOutlineThickness {
            pt: Pt((thickness_mm / PT_TO_MM) as f32),
        });
    }

    /// Stroke an empty circle. Caller must have set the outline colour and thickness via
    /// [`Canvas::set_stroke_black`] (or equivalent) at least once before the first call.
    pub fn circle_stroked(&mut self, cx_mm: f64, cy_mm: f64, r_mm: f64) {
        self.ops.push(Op::DrawPolygon {
            polygon: shapes::circle_polygon(cx_mm, cy_mm, r_mm, PaintMode::Stroke),
        });
    }

    /// Stroke a horizontal line from `(x1_mm, y_mm)` to `(x2_mm, y_mm)`.
    pub fn hline(&mut self, x1_mm: f64, x2_mm: f64, y_mm: f64) {
        self.ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point::new(Mm(x1_mm as f32), Mm(y_mm as f32)),
                        bezier: false,
                    },
                    LinePoint {
                        p: Point::new(Mm(x2_mm as f32), Mm(y_mm as f32)),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
    }

    /// Draw a single line of text whose left edge starts at `(x_mm, y_mm)`. Empty strings
    /// emit no ops.
    ///
    /// Each call wraps its own `StartTextSection` / `EndTextSection`. printpdf 0.9 emits
    /// `Op::SetTextCursor` as a PDF `Td` (relative move); without resetting the text
    /// matrix between calls, two cursors inside the same BT/ET *add* — the second
    /// position lands at `pos1 + pos2` instead of `pos2`. Wrapping per call resets the
    /// matrix at every BT, so every `text()` call lands at an absolute position.
    pub fn text(&mut self, x_mm: f64, y_mm: f64, s: &str, font: &FontId, size_pt: f64) {
        if s.is_empty() {
            return;
        }
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetFillColor { col: black() });
        self.ops.push(Op::SetFont {
            font: PdfFontHandle::External(font.clone()),
            size: Pt(size_pt as f32),
        });
        self.ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(x_mm as f32), Mm(y_mm as f32)),
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(s.to_string())],
        });
        self.ops.push(Op::EndTextSection);
    }

    /// Draw a single line of text whose RIGHT edge ends at `right_x_mm`. Width is
    /// estimated from char count using [`AVG_GLYPH_ADVANCE_RATIO`]; close enough for
    /// short Cyrillic + ASCII row labels (`Шифр-1`, `1`, `2.1.a`, …).
    pub fn text_right_aligned(
        &mut self,
        right_x_mm: f64,
        y_mm: f64,
        s: &str,
        font: &FontId,
        size_pt: f64,
    ) {
        if s.is_empty() {
            return;
        }
        let width_mm = estimate_text_width_mm(s, size_pt);
        self.text(right_x_mm - width_mm, y_mm, s, font, size_pt);
    }

    /// Draw a single line of text centred horizontally on `center_x_mm`. Width is
    /// estimated from char count using [`AVG_GLYPH_ADVANCE_RATIO`].
    pub fn text_centered_x(
        &mut self,
        center_x_mm: f64,
        y_mm: f64,
        s: &str,
        font: &FontId,
        size_pt: f64,
    ) {
        if s.is_empty() {
            return;
        }
        let width_mm = estimate_text_width_mm(s, size_pt);
        self.text(center_x_mm - width_mm / 2.0, y_mm, s, font, size_pt);
    }

    /// Draw a single character centred inside a circle of radius `r_mm` at
    /// `(cx_mm, cy_mm)`. Font size is auto-fitted to ~65% of the diameter — yields
    /// ~6 pt for a 3 mm bubble, matching the OMR-spec target range of 6–8 pt for
    /// in-bubble identifiers. A fully inked answer still covers the glyph at this size
    /// because the printed character lives in the glyph hull, not the whole bubble.
    pub fn text_centered_in_circle(
        &mut self,
        cx_mm: f64,
        cy_mm: f64,
        r_mm: f64,
        ch: char,
        font: &FontId,
    ) {
        let target_mm = r_mm * 2.0 * 0.65;
        let size_pt = target_mm / PT_TO_MM;
        let glyph_h_mm = size_pt * PT_TO_MM;
        // Approximate centring: shift left by ~30% of the glyph height and down by the
        // same so the baseline-anchored text sits roughly on the bubble centre.
        let x_mm = cx_mm - glyph_h_mm * 0.30;
        let y_mm = cy_mm - glyph_h_mm * 0.30;
        self.text(x_mm, y_mm, &ch.to_string(), font, size_pt);
    }
}

/// Rough text-width estimate in millimetres given character count and font size.
fn estimate_text_width_mm(s: &str, size_pt: f64) -> f64 {
    s.chars().count() as f64 * size_pt * AVG_GLYPH_ADVANCE_RATIO * PT_TO_MM
}

fn black() -> Color {
    Color::Rgb(Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        icc_profile: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use printpdf::{ParsedFont, PdfDocument};

    fn test_font() -> FontId {
        let bytes = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
        let mut warnings = Vec::new();
        let font = ParsedFont::from_bytes(bytes, 0, &mut warnings).expect("parse Noto Sans");
        let mut doc = PdfDocument::new("canvas-test");
        doc.add_font(&font)
    }

    fn show_text_count(ops: &[Op]) -> usize {
        ops.iter()
            .filter(|o| matches!(o, Op::ShowText { .. }))
            .count()
    }

    fn start_section_count(ops: &[Op]) -> usize {
        ops.iter()
            .filter(|o| matches!(o, Op::StartTextSection))
            .count()
    }

    #[test]
    fn text_skips_empty_string() {
        let mut c = Canvas::new();
        c.text(10.0, 10.0, "", &test_font(), 7.0);
        assert!(c.ops().is_empty());
    }

    #[test]
    fn text_wraps_each_call_in_its_own_bt_et() {
        let mut c = Canvas::new();
        c.text(10.0, 10.0, "A", &test_font(), 7.0);
        c.text(20.0, 10.0, "B", &test_font(), 7.0);
        c.text(30.0, 10.0, "C", &test_font(), 7.0);
        assert_eq!(show_text_count(c.ops()), 3);
        assert_eq!(start_section_count(c.ops()), 3);
    }

    #[test]
    fn text_right_aligned_uses_estimated_width() {
        let mut c = Canvas::new();
        c.text_right_aligned(50.0, 100.0, "ABC", &test_font(), 10.0);
        assert_eq!(show_text_count(c.ops()), 1);
        // Cursor should be left of the anchor; we don't assert the exact mm because the
        // width estimate is heuristic, but the X must be < 50 mm. `Point::x` is stored in
        // points (1 pt = 1/72 inch); 50 mm = 50 / 25.4 × 72 ≈ 141.73 pt.
        let cursor_x_pt = c.ops().iter().find_map(|op| match op {
            Op::SetTextCursor { pos } => Some(pos.x.0),
            _ => None,
        });
        let anchor_pt = (50.0_f32 / 25.4) * 72.0;
        assert!(matches!(cursor_x_pt, Some(x) if x < anchor_pt));
    }

    #[test]
    fn text_centered_in_circle_emits_one_show_text() {
        let mut c = Canvas::new();
        c.text_centered_in_circle(100.0, 100.0, 2.25, '5', &test_font());
        assert_eq!(show_text_count(c.ops()), 1);
    }

    #[test]
    fn circle_stroked_emits_one_polygon() {
        let mut c = Canvas::new();
        c.set_stroke_black(0.3);
        c.circle_stroked(50.0, 50.0, 2.25);
        let polygon_count = c
            .ops()
            .iter()
            .filter(|o| matches!(o, Op::DrawPolygon { .. }))
            .count();
        assert_eq!(polygon_count, 1);
    }
}
