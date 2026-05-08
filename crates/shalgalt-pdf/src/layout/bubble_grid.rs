//! Render a single [`BubbleGroup`] as one row of labelled circles.
//!
//! Acceptance defaults:
//! - 2.5 mm diameter, 0.4 mm outline, solid black.
//! - One-character label (A/B/C/D/E or 0–9) painted to the right of each bubble.
//! - Coordinates always go through [`crate::coords::project`] (margin-aware).

use printpdf::{Color, FontId, Mm, Op, PaintMode, PdfFontHandle, Point, Pt, Rgb, TextItem};
use shalgalt_core::domain::{template::BubbleGroup, PaperSpec};

use crate::{coords, shapes, style::BubbleStyle};

/// Draw a circle at every `group.bubbles[i]`. When `labels[i]` is present, paint a
/// single-character label to its right. Trailing bubbles whose label index is missing
/// stay unlabelled; an empty `labels` slice skips the label pass entirely.
pub fn draw(
    ops: &mut Vec<Op>,
    group: &BubbleGroup,
    paper: &PaperSpec,
    style: &BubbleStyle,
    labels: &[char],
    font: &FontId,
) {
    let r_mm = style.diameter_mm * 0.5;

    ops.push(Op::SetOutlineColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });
    ops.push(Op::SetOutlineThickness {
        pt: Pt((style.stroke_mm * 72.0 / 25.4) as f32),
    });

    // 1) Stroke every bubble — empty circles for the student to fill in.
    for bubble in &group.bubbles {
        let (cx_mm, cy_mm) = coords::project(bubble.x as f64, bubble.y as f64, paper);
        ops.push(Op::DrawPolygon {
            polygon: shapes::circle_polygon(
                cx_mm.0 as f64,
                cy_mm.0 as f64,
                r_mm,
                PaintMode::Stroke,
            ),
        });
    }

    // 2) Label pass.
    if labels.is_empty() {
        return;
    }
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font.clone()),
        size: Pt(style.label_size_pt as f32),
    });

    for (i, bubble) in group.bubbles.iter().enumerate() {
        let label = match labels.get(i) {
            Some(c) => *c,
            None => continue,
        };
        let (cx_mm, cy_mm) = coords::project(bubble.x as f64, bubble.y as f64, paper);
        // Place the label to the right of the bubble; nudge the baseline below the circle
        // centre for legibility.
        let label_x = cx_mm.0 as f64 + r_mm + style.label_offset_mm;
        let label_y = cy_mm.0 as f64 - style.label_size_pt * 0.35 / 2.83465; // approximate pt → mm correction
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(label_x as f32), Mm(label_y as f32)),
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(label.to_string())],
        });
    }

    ops.push(Op::EndTextSection);
}
