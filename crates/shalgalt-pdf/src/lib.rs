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

pub mod canvas;
pub mod coords;
mod error;
pub mod layout;
pub mod layout_map;
pub mod shapes;
pub mod style;

pub use canvas::Canvas;
pub use error::PdfError;
pub use layout_map::{BubbleEntry, LayoutMap, MarkerEntry};
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
    /// Draw a handwriting underline to the LEFT of every student-ID row so graders can
    /// fall back to a manual cipher read when OMR detection fails. `true` by default.
    pub manual_id_slots: bool,
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
            manual_id_slots: true,
            created_at: None,
        }
    }
}

/// Render the given [`OmrTemplate`] into a single-page PDF together with a
/// [`LayoutMap`] sidecar describing every printed bubble and marker in PDF
/// millimetres.
///
/// The grading pipeline (P3-02) reads coordinates from the [`LayoutMap`] rather than
/// re-projecting from the template, so renderer rounding cannot silently desynchronise
/// the grader from the printed sheet — see ADR 0007.
pub fn render_template_with_map(
    template: &OmrTemplate,
    opts: &PdfOptions,
) -> Result<(Vec<u8>, LayoutMap), PdfError> {
    let bytes = render_template(template, opts)?;
    let map = LayoutMap::from_template(template, opts);
    Ok((bytes, map))
}

/// Render the given [`OmrTemplate`] into a single-page PDF.
///
/// The returned `Vec<u8>` is a complete PDF byte stream from the `%PDF-` header through
/// the `%%EOF` marker, ready to write to disk or stream over HTTP. Use
/// [`render_template_with_map`] when the caller also needs the printed-bubble
/// coordinate sidecar (grading, scan correlation).
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

    let ops = build_page_ops(template, opts, &body_font_id, sidebar_font_id.as_ref());

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

/// Test-only re-export of [`build_page_ops`] so the regression test in
/// `tests/labels_layout.rs` can count `Op::ShowText` instances without parsing PDF
/// bytes. Hidden from the public API surface; do not call from production code.
#[doc(hidden)]
pub fn build_page_ops_for_test(
    template: &OmrTemplate,
    opts: &PdfOptions,
    body_font_id: &printpdf::FontId,
    sidebar_font_id: Option<&printpdf::FontId>,
) -> Vec<printpdf::Op> {
    build_page_ops(template, opts, body_font_id, sidebar_font_id)
}

/// Build the ordered list of drawing ops for a single page. Extracted from
/// [`render_template`] so the regression test in `tests/labels_layout.rs` can count
/// `Op::ShowText` instances without parsing PDF bytes.
pub(crate) fn build_page_ops(
    template: &OmrTemplate,
    opts: &PdfOptions,
    body_font_id: &printpdf::FontId,
    sidebar_font_id: Option<&printpdf::FontId>,
) -> Vec<printpdf::Op> {
    let mut canvas = Canvas::new();

    // 1) Four corner alignment markers.
    layout::markers::draw(&mut canvas, &template.markers, &opts.paper);

    // 2) Header (title / school / teacher). Fall back to `template.title` when the caller
    //    did not specify a title explicitly.
    let mut effective_header = opts.header.clone();
    if effective_header.title.is_none() && !template.title.is_empty() {
        effective_header.title = Some(template.title.clone());
    }
    layout::header::draw(
        &mut canvas,
        &effective_header,
        &template.groups,
        &opts.paper,
        body_font_id,
    );

    // 3) Handwriting-backup underlines for every student-ID row (Шифр backup). Drawn
    //    before the bubbles so any future overlay can paint on top.
    if opts.manual_id_slots {
        layout::manual_entry::draw(
            &mut canvas,
            &template.groups,
            &opts.paper,
            &opts.bubble_style,
        );
    }

    // 3b) Section headers (1-Р ХЭСЭГ / 2-Р ХЭСЭГ + 2.1 / 2.2 / 2.3 / 2.4 stickers).
    //     Шифр and Вариант suppress their headers — the layout already reads as cipher
    //     and variant at a glance.
    layout::section_headers::draw(
        &mut canvas,
        &template.groups,
        &opts.paper,
        &opts.bubble_style,
        body_font_id,
    );

    // 4) Walk every group: row label to the left, then bubbles + per-bubble labels
    //    drawn INSIDE each circle. Choice/digit labels come from `PdfOptions`. We pick
    //    digit labels for any group whose bubble count exceeds `choice_labels.len()` so
    //    numeric-question rows (kind=Question, 10 bubbles) get 0–9 instead of running
    //    out of A–E. `BubbleKind::StudentId` always uses digit labels.
    for group in &template.groups {
        let labels: &[char] = match group.kind {
            shalgalt_core::domain::BubbleKind::StudentId => &opts.digit_labels,
            // The variant selector is a single short choice row (A/B/C/…), so it
            // takes the same A–E choice labels as a question.
            shalgalt_core::domain::BubbleKind::Variant => &opts.choice_labels,
            shalgalt_core::domain::BubbleKind::Question => {
                if group.bubbles.len() > opts.choice_labels.len() {
                    &opts.digit_labels
                } else {
                    &opts.choice_labels
                }
            }
        };

        layout::labels::draw_row_label(
            &mut canvas,
            group,
            &opts.paper,
            &opts.bubble_style,
            body_font_id,
        );
        layout::bubble_grid::draw(
            &mut canvas,
            group,
            &opts.paper,
            &opts.bubble_style,
            labels,
            body_font_id,
        );
    }

    // 5) Optional variant tag (`[A]` / `[B]`) in the upper-right corner.
    if let Some(label) = opts.variant.as_deref() {
        push_variant_label(&mut canvas, label, &opts.paper, body_font_id);
    }

    // 6) Sidebar (90° rotation). The default instructions string is Mongolian Cyrillic,
    //    which NotoSans renders. Only switch to `sidebar_font_id` (NotoSansMongolian) when
    //    a future caller passes a string in traditional Mongolian script.
    if let Some(text) = opts.instructions.as_deref() {
        // Force the body font for Cyrillic instructions; the Mongolian-script font is
        // embedded but unused until traditional script support lands.
        let _ = sidebar_font_id;
        layout::sidebar::draw(&mut canvas, text, &opts.paper, body_font_id);
    }

    // The P5 answer-key overlay branch will land alongside `#P5-01`.
    let _ = opts.include_answer_key_overlay;

    canvas.into_ops()
}

/// Print a variant label (`[A]`, `[B]`, …) in the upper-right corner. Only invoked when
/// the caller wants to flag a variant on the printed sheet.
fn push_variant_label(
    canvas: &mut Canvas,
    label: &str,
    paper: &PaperSpec,
    font: &printpdf::FontId,
) {
    let baseline_y_mm = paper.height_mm - paper.margin_mm - 8.0;
    let x_mm = paper.width_mm - paper.margin_mm - 18.0;
    canvas.text(x_mm, baseline_y_mm, &format!("[{label}]"), font, 14.0);
}
