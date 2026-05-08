//! Top-of-page header block. Prints the exam title plus optional subtitle, school, and
//! teacher lines using the Latin + Cyrillic body font (Noto Sans).
//!
//! Every string arrives pre-translated via [`HeaderText`]. The renderer never imports an
//! i18n table.

use printpdf::{Color, FontId, Mm, Op, PdfFontHandle, Point, Pt, Rgb, TextItem};
use shalgalt_core::domain::PaperSpec;

use crate::style::HeaderText;

/// Draw the header block. An empty [`HeaderText`] adds no ops.
pub fn draw(ops: &mut Vec<Op>, header: &HeaderText, paper: &PaperSpec, font: &FontId) {
    if header == &HeaderText::default() {
        return;
    }

    let baseline_top_mm = paper.height_mm - paper.margin_mm - 12.0;
    let x_mm = paper.margin_mm;

    // One text section, lines stacked via AddLineBreak.
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(x_mm as f32), Mm(baseline_top_mm as f32)),
    });
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });
    ops.push(Op::SetLineHeight { lh: Pt(14.0) });

    // Title — 18 pt.
    if let Some(title) = header.title.as_deref() {
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font.clone()),
            size: Pt(18.0),
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(title.to_string())],
        });
        ops.push(Op::AddLineBreak);
    }

    // Remaining lines drop to 12 pt subtitle / school / teacher.
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font.clone()),
        size: Pt(12.0),
    });
    ops.push(Op::SetLineHeight { lh: Pt(12.0) });

    let mut sub_lines: Vec<&str> = Vec::new();
    if let Some(s) = header.subtitle.as_deref() {
        sub_lines.push(s);
    }
    if let Some(s) = header.school.as_deref() {
        sub_lines.push(s);
    }
    if let Some(s) = header.teacher.as_deref() {
        sub_lines.push(s);
    }

    for line in sub_lines {
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(line.to_string())],
        });
        ops.push(Op::AddLineBreak);
    }

    ops.push(Op::EndTextSection);
}
