//! Right-edge vertical sidebar. Prints student-facing instructions rotated 90°.
//!
//! Uses [`NotoSansMongolian-Regular.ttf`] rather than the Latin/Cyrillic body font; the
//! caller passes its `FontId` in here, while the actual font embedding is owned by
//! [`crate::render_template`].
//!
//! Rotation is implemented via `Op::SetTextMatrix(TextMatrix::TranslateRotate(x, y, deg))`,
//! which maps directly onto `printpdf` 0.9's `Tm` operator. The whole sequence is wrapped
//! in `SaveGraphicsState` / `RestoreGraphicsState` so neighbouring ops are unaffected.

use printpdf::{Color, FontId, Op, PdfFontHandle, Pt, Rgb, TextItem, TextMatrix};
use shalgalt_core::domain::PaperSpec;

/// Paint `text` onto the right-edge vertical strip. An empty string adds no ops.
pub fn draw(ops: &mut Vec<Op>, text: &str, paper: &PaperSpec, font: &FontId) {
    if text.is_empty() {
        return;
    }

    // Sidebar baseline anchor: 8 mm inside the right margin, vertically starting just
    // below the page midpoint and extending upward (i.e. positive x in the rotated frame).
    let x_mm = (paper.width_mm - paper.margin_mm - 8.0) as f32;
    let y_mm = (paper.height_mm * 0.5 - 60.0) as f32;

    ops.push(Op::SaveGraphicsState);
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
        size: Pt(11.0),
    });
    // Counter-clockwise 90° rotation + absolute start position. `TranslateRotate` operates
    // on page-absolute coordinates.
    ops.push(Op::SetTextMatrix {
        matrix: TextMatrix::TranslateRotate(
            printpdf::Pt(x_mm * 72.0 / 25.4),
            printpdf::Pt(y_mm * 72.0 / 25.4),
            90.0,
        ),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(text.to_string())],
    });
    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);
}
