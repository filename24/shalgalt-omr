//! `shalgalt-pdf` — pure-Rust OMR PDF renderer backed by [`printpdf`].
//!
//! P2-05 scope: bring the crate online — embed both Noto Sans fonts, accept an
//! [`OmrTemplate`], and produce a valid one-page PDF whose only painted content is the
//! four corner reference markers and the template title (the latter exists so the
//! Cyrillic font pipeline is exercised by the test suite).
//!
//! The full OMR layout (Шифр / variant / multi-choice grid / numeric / sidebar) lands
//! in P2-06 on top of this scaffold. The crate stays free of `tauri` / OpenCV / pdfium
//! deps so it is reusable from `apps/server` and future CLI tools.
//!
//! # Coordinates
//!
//! Template coordinates are normalized to `[0, 1]` (master plan §9.2). PDF pages use
//! `printpdf`'s native bottom-left origin in millimetres. The translation lives in
//! [`coords::to_page_mm`] and is shared with P2-06.
//!
//! # Determinism
//!
//! `PdfSaveOptions::subset_fonts` is enabled so the output only embeds the glyphs
//! actually drawn — keeping byte output stable enough for the golden-file tests
//! introduced in P2-12.

use printpdf::{
    Color, FontId, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle, PdfFontParseWarning,
    PdfPage, PdfSaveOptions, PdfWarnMsg, Point, Polygon, PolygonRing, Pt, Rgb, TextItem,
    WindingOrder,
};
use shalgalt_core::domain::{
    template::{Marker, OmrTemplate},
    PaperSpec,
};

pub mod coords;
mod error;

pub use error::PdfError;

/// Re-export the domain types this renderer consumes so callers do not need to depend on
/// `shalgalt-core` directly.
pub mod domain {
    pub use shalgalt_core::domain::paper::{Orientation, PaperSpec};
    pub use shalgalt_core::domain::template::{
        BubbleGroup, BubbleKind, Marker, OmrTemplate, TemplatePoint,
    };
}

/// Latin + Cyrillic (Mongolian Cyrillic) body font. Embedded into the binary at compile
/// time via `include_bytes!`.
const NOTO_SANS_REGULAR: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");

/// Traditional Mongolian script fallback font. The P2-05 skeleton only embeds it; P2-06
/// (sidebar) and P4 (project file) wire it into the actual draw calls. Kept around so the
/// binary always carries it.
#[allow(dead_code)]
const NOTO_SANS_MONGOLIAN_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/NotoSansMongolian-Regular.ttf");

/// Caller-controlled switches for a single-page render.
///
/// `paper` decides the page rectangle and margin (in mm). `variant` is the label printed
/// on multi-variant exams (`A` / `B` / …) — `None` for single-variant tests.
/// `include_answer_key_overlay` is the P5 hook for teacher-facing proof prints; ignored
/// in the P2-05 skeleton.
#[derive(Debug, Clone)]
pub struct PdfOptions {
    /// Sheet geometry.
    pub paper: PaperSpec,
    /// Variant label (`"A"` / `"B"`). `None` for single-variant exams.
    pub variant: Option<String>,
    /// When `true`, draws the teacher answer-key overlay (enabled in P5).
    pub include_answer_key_overlay: bool,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            paper: PaperSpec::A4_PORTRAIT,
            variant: None,
            include_answer_key_overlay: false,
        }
    }
}

/// Render the given [`OmrTemplate`] into a single-page PDF.
///
/// The returned `Vec<u8>` is a complete PDF byte stream from the `%PDF-` header through
/// the `%%EOF` marker, ready to write to disk or stream over HTTP.
pub fn render_template(template: &OmrTemplate, opts: &PdfOptions) -> Result<Vec<u8>, PdfError> {
    let mut doc = PdfDocument::new(&template.title);

    // Embed the Latin + Cyrillic body font. `from_bytes` uses font index 0 (we ship a
    // single-face TTF, not a TTC).
    let mut font_warnings: Vec<PdfFontParseWarning> = Vec::new();
    let body_font = ParsedFont::from_bytes(NOTO_SANS_REGULAR, 0, &mut font_warnings).ok_or(
        PdfError::FontLoad {
            font: "NotoSans-Regular",
            message: "ParsedFont::from_bytes returned None".into(),
        },
    )?;
    let body_font_id = doc.add_font(&body_font);

    let mut ops: Vec<Op> = Vec::new();

    // 1) Four corner alignment markers (placeholder — P2-06 may swap in ArUco).
    push_marker_squares(&mut ops, &template.markers, &opts.paper);

    // 2) Print the exam title at the top of the page (inside the margin). This minimal
    //    text output exercises the Cyrillic / Mongolian glyph subset pipeline at P2-05
    //    acceptance time — `printpdf` would fail at the subsetting stage if a glyph is
    //    missing.
    push_title(&mut ops, &template.title, &opts.paper, &body_font_id);

    // 3) Stamp the variant label in the upper-right corner so printed copies cannot be
    //    mixed up post-hoc.
    if let Some(label) = opts.variant.as_deref() {
        push_variant_label(&mut ops, label, &opts.paper, &body_font_id);
    }

    // The P5 answer-key overlay is not wired up yet. Once P2-06 starts using the
    // template's answer indices for real, we add the overlay branch here.
    let _ = opts.include_answer_key_overlay;

    let page = PdfPage::new(
        Mm(opts.paper.width_mm as f32),
        Mm(opts.paper.height_mm as f32),
        ops,
    );

    let save_opts = PdfSaveOptions {
        subset_fonts: true,
        ..Default::default()
    };
    let mut save_warnings: Vec<PdfWarnMsg> = Vec::new();
    let bytes = doc
        .with_pages(vec![page])
        .save(&save_opts, &mut save_warnings);

    if !bytes.starts_with(b"%PDF-") {
        return Err(PdfError::Encode {
            reason: "printpdf produced output without a %PDF- header".into(),
        });
    }

    Ok(bytes)
}

/// Stamp the four markers as solid squares. Coordinates go through the normalized → mm
/// helper.
fn push_marker_squares(ops: &mut Vec<Op>, markers: &[Marker; 4], paper: &PaperSpec) {
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });

    for marker in markers {
        let (cx, cy) =
            coords::to_page_mm(marker.position.x as f64, marker.position.y as f64, paper);
        // The marker's `size` is normalized; convert to mm using the shorter page edge
        // and use the half-extent below.
        let half = marker.size as f64 * paper.width_mm.min(paper.height_mm) * 0.5;
        ops.push(Op::DrawPolygon {
            polygon: square_polygon_around(cx, cy, half),
        });
    }
}

/// Print the title at the top-left of the page (inside the margin plus a small offset).
fn push_title(ops: &mut Vec<Op>, title: &str, paper: &PaperSpec, body_font_id: &FontId) {
    if title.is_empty() {
        return;
    }

    // Baseline sits 12 mm below the top margin.
    let baseline_y_mm = paper.height_mm - paper.margin_mm - 12.0;
    let x_mm = paper.margin_mm;

    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(x_mm as f32), Mm(baseline_y_mm as f32)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(body_font_id.clone()),
        size: Pt(18.0),
    });
    ops.push(Op::SetLineHeight { lh: Pt(20.0) });
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(title.to_string())],
    });
    ops.push(Op::EndTextSection);
}

/// Print a variant label (`[A]`, `[B]`, …) in the upper-right corner.
fn push_variant_label(ops: &mut Vec<Op>, label: &str, paper: &PaperSpec, body_font_id: &FontId) {
    let baseline_y_mm = paper.height_mm - paper.margin_mm - 8.0;
    let x_mm = paper.width_mm - paper.margin_mm - 18.0;

    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(x_mm as f32), Mm(baseline_y_mm as f32)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(body_font_id.clone()),
        size: Pt(14.0),
    });
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(format!("[{label}]"))],
    });
    ops.push(Op::EndTextSection);
}

/// Polygon for a square of side `2 * half` mm, centered at `(cx, cy)` mm.
fn square_polygon_around(cx: f64, cy: f64, half: f64) -> Polygon {
    let pts = [
        (cx - half, cy - half),
        (cx + half, cy - half),
        (cx + half, cy + half),
        (cx - half, cy + half),
    ];
    Polygon {
        rings: vec![PolygonRing {
            points: pts
                .iter()
                .map(|(x, y)| printpdf::LinePoint {
                    p: Point::new(Mm(*x as f32), Mm(*y as f32)),
                    bezier: false,
                })
                .collect(),
        }],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    }
}
