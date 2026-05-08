//! `shalgalt-pdf` — pure-Rust OMR PDF renderer backed by [`printpdf`].
//!
//! P2-05 brought the crate online with embedded Noto Sans fonts and a `render_template`
//! shell that drew corner markers + title. P2-06 turns the output into an actually
//! print-ready OMR card:
//!
//! - four corner alignment markers ([`layout::markers`])
//! - exam title / school / teacher header ([`layout::header`])
//! - per-`BubbleGroup` labelled circles ([`layout::bubble_grid`])
//! - vertical numeric blocks (Шифр / Section-2) ([`layout::numeric_block`])
//! - right-edge 90°-rotated instructions sidebar ([`layout::sidebar`])
//!
//! The crate stays free of `tauri` / OpenCV / pdfium deps so it is reusable from
//! `apps/server` and future CLI tools.
//!
//! # Coordinates
//!
//! Template coordinates are normalized to `[0, 1]` (master plan §9.2). PDF pages use
//! `printpdf`'s native bottom-left origin in millimetres. The translation lives in
//! [`coords`] — both [`coords::to_page_mm`] (full page) for markers and
//! [`coords::project`] (margin-aware) for body content.
//!
//! # Determinism
//!
//! - `PdfSaveOptions::subset_fonts` is enabled so the output only embeds the glyphs
//!   actually drawn.
//! - `PdfDocument::new` defaults the `creation_date` / `modification_date` to the unix
//!   epoch — fixed across runs unless the caller overrides via [`PdfOptions::created_at`].
//! - The PDF trailer's `/ID` array is randomized per run by `printpdf`. The golden-file
//!   test in `tests/golden.rs` strips `/ID` before comparison.

use printpdf::{
    DateTime, ParsedFont, PdfDocument, PdfFontParseWarning, PdfPage, PdfSaveOptions, PdfWarnMsg,
};
use shalgalt_core::domain::{template::OmrTemplate, PaperSpec};

pub mod coords;
mod error;
pub mod layout;
pub mod shapes;
pub mod style;

pub use error::PdfError;
pub use style::{BubbleStyle, HeaderText};

/// Re-export the domain types this renderer consumes so callers do not need to depend on
/// `shalgalt-core` directly.
pub mod domain {
    pub use shalgalt_core::domain::paper::{Orientation, PaperSpec};
    pub use shalgalt_core::domain::template::{
        BubbleGroup, BubbleKind, Marker, MarkerKind, OmrTemplate, TemplatePoint,
    };
}

/// Latin + Cyrillic (Mongolian Cyrillic) body font. Embedded into the binary at compile
/// time via `include_bytes!`.
const NOTO_SANS_REGULAR: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");

/// Traditional Mongolian script fallback font for the sidebar. Latin Noto Sans cannot
/// render the Mongolian verticals, so any sidebar text always uses this font.
const NOTO_SANS_MONGOLIAN_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/NotoSansMongolian-Regular.ttf");

/// Caller-controlled switches for a single-page render.
///
/// All text input is expected to be pre-translated — this crate does not know about the
/// i18n table.
#[derive(Debug, Clone)]
pub struct PdfOptions {
    /// Sheet geometry.
    pub paper: PaperSpec,
    /// Variant label (`"A"` / `"B"`). `None` for single-variant exams.
    pub variant: Option<String>,
    /// When `true`, draws the teacher answer-key overlay (enabled in P5).
    pub include_answer_key_overlay: bool,
    /// Pre-translated header text. When all fields are `None`, the renderer falls back to
    /// `template.title` only.
    pub header: HeaderText,
    /// Sidebar instructions in Mongolian script. `None` skips the sidebar entirely.
    pub instructions: Option<String>,
    /// Multiple-choice option labels (e.g. `['A','B','C','D','E']`). Empty disables labels.
    pub choice_labels: Vec<char>,
    /// Numeric-block labels (Шифр / Section-2), e.g. `['0','1', …, '9']`.
    pub digit_labels: Vec<char>,
    /// Bubble visual style.
    pub bubble_style: BubbleStyle,
    /// Explicit override for `/CreationDate` and `/ModDate`. `None` keeps the printpdf
    /// epoch default — also deterministic.
    pub created_at: Option<DateTime>,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            paper: PaperSpec::A4_PORTRAIT,
            variant: None,
            include_answer_key_overlay: false,
            header: HeaderText::default(),
            instructions: None,
            choice_labels: vec!['A', 'B', 'C', 'D', 'E'],
            digit_labels: vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
            bubble_style: BubbleStyle::default(),
            created_at: None,
        }
    }
}

/// Render the given [`OmrTemplate`] into a single-page PDF.
///
/// The returned `Vec<u8>` is a complete PDF byte stream from the `%PDF-` header through
/// the `%%EOF` marker, ready to write to disk or stream over HTTP.
pub fn render_template(template: &OmrTemplate, opts: &PdfOptions) -> Result<Vec<u8>, PdfError> {
    let mut doc = PdfDocument::new(&template.title);

    // Pin the document timestamps when the caller asks for an explicit value.
    if let Some(ts) = opts.created_at {
        doc.metadata.info.creation_date = ts;
        doc.metadata.info.modification_date = ts;
        doc.metadata.info.metadata_date = ts;
    }

    // Embed the Latin + Cyrillic body font.
    let mut font_warnings: Vec<PdfFontParseWarning> = Vec::new();
    let body_font = ParsedFont::from_bytes(NOTO_SANS_REGULAR, 0, &mut font_warnings).ok_or(
        PdfError::FontLoad {
            font: "NotoSans-Regular",
            message: "ParsedFont::from_bytes returned None".into(),
        },
    )?;
    let body_font_id = doc.add_font(&body_font);

    // Only embed the Mongolian-script font when the caller actually has sidebar text —
    // skipping it keeps the output bytes small for the common case.
    let sidebar_font_id = if opts.instructions.is_some() {
        let mut warns: Vec<PdfFontParseWarning> = Vec::new();
        let mongolian_font = ParsedFont::from_bytes(NOTO_SANS_MONGOLIAN_REGULAR, 0, &mut warns)
            .ok_or(PdfError::FontLoad {
                font: "NotoSansMongolian-Regular",
                message: "ParsedFont::from_bytes returned None".into(),
            })?;
        Some(doc.add_font(&mongolian_font))
    } else {
        None
    };

    let mut ops: Vec<printpdf::Op> = Vec::new();

    // 1) Four corner alignment markers.
    layout::markers::draw(&mut ops, &template.markers, &opts.paper);

    // 2) Header (title / school / teacher). Fall back to `template.title` when the caller
    //    did not specify a title explicitly.
    let mut effective_header = opts.header.clone();
    if effective_header.title.is_none() && !template.title.is_empty() {
        effective_header.title = Some(template.title.clone());
    }
    layout::header::draw(&mut ops, &effective_header, &opts.paper, &body_font_id);

    // 3) Render every BubbleGroup. The label set depends on the group's kind.
    for group in &template.groups {
        let labels: &[char] = match group.kind {
            shalgalt_core::domain::BubbleKind::StudentId => &opts.digit_labels,
            shalgalt_core::domain::BubbleKind::Question => &opts.choice_labels,
        };
        layout::bubble_grid::draw(
            &mut ops,
            group,
            &opts.paper,
            &opts.bubble_style,
            labels,
            &body_font_id,
        );
    }

    // 4) Optional variant tag (`[A]` / `[B]`) in the upper-right corner.
    if let Some(label) = opts.variant.as_deref() {
        push_variant_label(&mut ops, label, &opts.paper, &body_font_id);
    }

    // 5) Sidebar (Mongolian script, 90° rotation).
    if let (Some(text), Some(font)) = (opts.instructions.as_deref(), sidebar_font_id.as_ref()) {
        layout::sidebar::draw(&mut ops, text, &opts.paper, font);
    }

    // The P5 answer-key overlay branch will land alongside `#P5-01`.
    let _ = opts.include_answer_key_overlay;

    let page = PdfPage::new(
        printpdf::Mm(opts.paper.width_mm as f32),
        printpdf::Mm(opts.paper.height_mm as f32),
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

/// Print a variant label (`[A]`, `[B]`, …) in the upper-right corner. Only invoked when
/// the caller wants to flag a variant on the printed sheet.
fn push_variant_label(
    ops: &mut Vec<printpdf::Op>,
    label: &str,
    paper: &PaperSpec,
    body_font_id: &printpdf::FontId,
) {
    use printpdf::{Color, Mm, Op, PdfFontHandle, Point, Pt, Rgb, TextItem};

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
