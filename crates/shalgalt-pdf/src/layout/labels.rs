//! Per-group row labels drawn to the LEFT of the first bubble.
//!
//! The choice/digit letters live INSIDE each bubble (see [`crate::layout::bubble_grid`]),
//! so the only label this module emits is the row identifier — `"1"`, `"2"`, … for
//! Section 1 questions, `"a"`, `"b"`, … for Section 2 numeric rows, `"Шифр-1"` etc. for
//! the cipher block. Pulled from [`BubbleGroup::label`].
//!
//! The label text is *right-aligned* against the bubble: callers want the label hugging
//! the first circle without bleeding into it. [`Canvas::text_right_aligned`] approximates
//! width from char count — accurate enough for short Cyrillic + ASCII labels.

use printpdf::FontId;
use shalgalt_core::domain::{template::BubbleGroup, PaperSpec};

use crate::{canvas::Canvas, coords, style::BubbleStyle};

/// Approximate `pt → mm` correction. 1 pt = 1/72 inch; 1 inch = 25.4 mm.
const PT_TO_MM: f64 = 25.4 / 72.0;

/// Gap (mm) between the right edge of the label text and the left edge of the first
/// bubble circle. Smaller than [`BubbleStyle::row_label_offset_mm`] because we now
/// right-align the text instead of left-anchoring it.
const LABEL_BUBBLE_GAP_MM: f64 = 1.5;

/// Draw [`BubbleGroup::label`] right-aligned to the LEFT of the group's first bubble.
/// Skips when the label is empty so a roster of unlabelled groups produces no stray text.
pub fn draw_row_label(
    canvas: &mut Canvas,
    group: &BubbleGroup,
    paper: &PaperSpec,
    style: &BubbleStyle,
    font: &FontId,
) {
    let label = group.label.trim();
    if label.is_empty() {
        return;
    }
    let Some(anchor) = group.bubbles.first() else {
        return;
    };

    let r_mm = style.diameter_mm * 0.5;
    let (cx_mm, cy_mm) = coords::project(anchor.x as f64, anchor.y as f64, paper);
    let right_x = cx_mm.0 as f64 - r_mm - LABEL_BUBBLE_GAP_MM;
    let baseline_y = cy_mm.0 as f64 - style.label_size_pt * 0.35 * PT_TO_MM;

    canvas.text_right_aligned(right_x, baseline_y, label, font, style.label_size_pt);
}

#[cfg(test)]
mod tests {
    use super::*;
    use printpdf::{Op, ParsedFont, PdfDocument};
    use shalgalt_core::domain::template::{BubbleKind, TemplatePoint};

    fn test_font() -> FontId {
        let bytes = include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");
        let mut warnings = Vec::new();
        let font = ParsedFont::from_bytes(bytes, 0, &mut warnings).expect("parse Noto Sans");
        let mut doc = PdfDocument::new("labels-test");
        doc.add_font(&font)
    }

    fn dummy_group(label: &str, bubble_count: usize) -> BubbleGroup {
        BubbleGroup {
            id: "g-test".to_string(),
            kind: BubbleKind::Question,
            label: label.to_string(),
            bubbles: (0..bubble_count)
                .map(|i| TemplatePoint {
                    x: 0.1 + i as f32 * 0.05,
                    y: 0.5,
                })
                .collect(),
            answer_index: None,
            score: 1.0,
            section: None,
        }
    }

    fn show_text_count(ops: &[Op]) -> usize {
        ops.iter()
            .filter(|o| matches!(o, Op::ShowText { .. }))
            .count()
    }

    #[test]
    fn draw_row_label_skips_empty_label() {
        let mut canvas = Canvas::new();
        draw_row_label(
            &mut canvas,
            &dummy_group("", 4),
            &PaperSpec::A4_PORTRAIT,
            &BubbleStyle::default(),
            &test_font(),
        );
        assert_eq!(show_text_count(canvas.ops()), 0);
    }

    #[test]
    fn draw_row_label_skips_when_group_has_no_bubbles() {
        let mut canvas = Canvas::new();
        draw_row_label(
            &mut canvas,
            &dummy_group("Q1", 0),
            &PaperSpec::A4_PORTRAIT,
            &BubbleStyle::default(),
            &test_font(),
        );
        assert_eq!(show_text_count(canvas.ops()), 0);
    }

    #[test]
    fn draw_row_label_emits_one_op_for_non_empty_label() {
        let mut canvas = Canvas::new();
        draw_row_label(
            &mut canvas,
            &dummy_group("Q1", 4),
            &PaperSpec::A4_PORTRAIT,
            &BubbleStyle::default(),
            &test_font(),
        );
        assert_eq!(show_text_count(canvas.ops()), 1);
    }
}
