//! САНАМЖ — instructions block in the top-right of the page.
//!
//! Per [`docs/MONGOLIAN_OMR_SPEC.md`](../../../../docs/MONGOLIAN_OMR_SPEC.md) §2.4 the
//! instructions are horizontal multi-line text in the top-right zone, NOT the rotated
//! sidebar of the previous prototype. The string arrives pre-translated via
//! [`PdfOptions::instructions`]; this module splits on sentence boundaries and word-wraps
//! to a fixed column width.
//!
//! The font handle defaults to the body font (Cyrillic). Traditional Mongolian script
//! support lands when the i18n table can hand a vertical-script string to the renderer
//! and the caller passes a different `FontId`.

use printpdf::{Color, FontId, Mm, Op, PdfFontHandle, Point, Pt, Rgb, TextItem};
use shalgalt_core::domain::PaperSpec;

use crate::canvas::Canvas;

/// Header strap printed before the body lines.
const HEADER_TEXT: &str = "САНАМЖ";

/// Header font size.
const HEADER_FONT_PT: f64 = 12.0;

/// Body font size — 10 pt for unaided readability per the OMR spec.
const BODY_FONT_PT: f64 = 10.0;

/// Distance between consecutive body lines (point units).
const LINE_HEIGHT_PT: f64 = 12.0;

/// X position (normalized 0–1) where the instructions block starts. Aligned with
/// Section 2's `startX` (`0.55`) so the САНАМЖ column and the answer-area numeric
/// blocks share a single right-half grid line. The 14 pt page title (≈ 68 mm wide)
/// ends ~30 mm to the left of this anchor, so no bleed risk.
const ANCHOR_X_NORMALIZED: f64 = 0.55;

/// Y position of the header baseline relative to the page top, in mm.
const HEADER_Y_OFFSET_FROM_TOP_MM: f64 = 22.0;

/// Approximate `pt → mm` correction. 1 pt = 1/72 inch; 1 inch = 25.4 mm.
const PT_TO_MM: f64 = 25.4 / 72.0;

/// Average glyph advance / font-size ratio used to estimate line widths during wrapping.
const AVG_GLYPH_ADVANCE_RATIO: f64 = 0.55;

/// Paint САНАМЖ + the instructions text into the top-right zone. Empty input adds no ops.
pub fn draw(canvas: &mut Canvas, text: &str, paper: &PaperSpec, font: &FontId) {
    if text.trim().is_empty() {
        return;
    }

    let header_x = paper.width_mm * ANCHOR_X_NORMALIZED;
    let header_y = paper.height_mm - HEADER_Y_OFFSET_FROM_TOP_MM;

    // Header strap.
    canvas.text(header_x, header_y, HEADER_TEXT, font, HEADER_FONT_PT);

    // Split on hard line breaks first so numbered items ("1. ...", "2. ...") each
    // start a fresh line, then word-wrap each paragraph to the column width.
    let max_x = paper.width_mm - paper.margin_mm;
    let max_width_mm = max_x - header_x;
    let wrapped: Vec<String> = text
        .split('\n')
        .flat_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                vec![String::new()]
            } else {
                wrap_paragraph(trimmed, max_width_mm, BODY_FONT_PT)
            }
        })
        .collect();

    // Body lines emitted in a single BT/ET block via AddLineBreak so the line-height
    // stays consistent across paragraphs.
    let body_baseline_y = header_y - HEADER_FONT_PT * PT_TO_MM - 4.0;
    canvas.push(Op::StartTextSection);
    canvas.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });
    canvas.push(Op::SetFont {
        font: PdfFontHandle::External(font.clone()),
        size: Pt(BODY_FONT_PT as f32),
    });
    canvas.push(Op::SetLineHeight {
        lh: Pt(LINE_HEIGHT_PT as f32),
    });
    canvas.push(Op::SetTextCursor {
        pos: Point::new(Mm(header_x as f32), Mm(body_baseline_y as f32)),
    });

    let mut first = true;
    for line in &wrapped {
        if !first {
            canvas.push(Op::AddLineBreak);
        }
        first = false;
        canvas.push(Op::ShowText {
            items: vec![TextItem::Text(line.clone())],
        });
    }

    canvas.push(Op::EndTextSection);
}

/// Word-wrap `text` so each emitted line fits inside `max_width_mm` at `size_pt`.
/// Splits on whitespace; greedy fill. Estimates width from `AVG_GLYPH_ADVANCE_RATIO`,
/// which is good enough for short Cyrillic + ASCII instruction strings.
fn wrap_paragraph(text: &str, max_width_mm: f64, size_pt: f64) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let width = estimate_width_mm(&candidate, size_pt);
        if width <= max_width_mm {
            current = candidate;
        } else if current.is_empty() {
            // Single word longer than column — emit it on its own line and let it
            // overflow rather than splitting mid-glyph.
            lines.push(word.to_string());
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn estimate_width_mm(s: &str, size_pt: f64) -> f64 {
    s.chars().count() as f64 * size_pt * AVG_GLYPH_ADVANCE_RATIO * PT_TO_MM
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_paragraph_empty_input_yields_no_lines() {
        assert!(wrap_paragraph("", 50.0, 9.0).is_empty());
    }

    #[test]
    fn wrap_paragraph_short_input_fits_on_one_line() {
        let lines = wrap_paragraph("Зөв хариулт", 50.0, 9.0);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn wrap_paragraph_long_input_splits_into_multiple_lines() {
        let text = "Зөв хариултыг бөглөнө үү. Зөвхөн хар балаар бөглөнө. \
                    Будахдаа бүхэл дугуйг бүтнээр бөглөнө үү.";
        let lines = wrap_paragraph(text, 50.0, 9.0);
        assert!(lines.len() >= 2);
    }
}
