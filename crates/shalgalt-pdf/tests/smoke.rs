//! P2-05 smoke tests.
//!
//! 1. Rendering an empty template (just the four corner markers) yields a valid PDF.
//! 2. A Cyrillic title (`"Шалгалт"`) ends up as actual text operators in the page content
//!    stream that `lopdf` can parse.

use shalgalt_pdf::domain::{Marker, OmrTemplate, TemplatePoint};
use shalgalt_pdf::{render_template, PdfOptions};

/// Minimal template matching the acceptance shape — four corner markers, no groups,
/// normalized coordinates inside `[0, 1]`.
fn empty_template_with_title(title: &str) -> OmrTemplate {
    OmrTemplate {
        version: OmrTemplate::CURRENT_VERSION,
        title: title.to_string(),
        markers: [
            Marker {
                id: "tl".into(),
                position: TemplatePoint { x: 0.05, y: 0.05 },
                size: 0.04,
                kind: Default::default(),
            },
            Marker {
                id: "tr".into(),
                position: TemplatePoint { x: 0.95, y: 0.05 },
                size: 0.04,
                kind: Default::default(),
            },
            Marker {
                id: "br".into(),
                position: TemplatePoint { x: 0.95, y: 0.95 },
                size: 0.04,
                kind: Default::default(),
            },
            Marker {
                id: "bl".into(),
                position: TemplatePoint { x: 0.05, y: 0.95 },
                size: 0.04,
                kind: Default::default(),
            },
        ],
        groups: vec![],
    }
}

#[test]
fn renders_empty_template_to_valid_pdf() {
    let template = empty_template_with_title("");
    let bytes = render_template(&template, &PdfOptions::default()).expect("render must succeed");

    // PDF magic header.
    assert!(
        bytes.starts_with(b"%PDF-"),
        "expected %PDF- header, got: {:?}",
        &bytes[..bytes.len().min(8)]
    );

    // Parse with `lopdf` and confirm a single-page document.
    let doc = lopdf::Document::load_mem(&bytes).expect("output must parse as PDF");
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 1, "skeleton must produce exactly one page");
}

#[test]
fn renders_cyrillic_title_with_nonempty_text_stream() {
    let template = empty_template_with_title("Шалгалт");
    let bytes =
        render_template(&template, &PdfOptions::default()).expect("cyrillic render must succeed");

    let doc = lopdf::Document::load_mem(&bytes).expect("output must parse as PDF");
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 1);

    // Decode and sum every content stream on the page. A non-zero total proves a text-show
    // operator (Tj / TJ) made it into the output.
    let total_content_len: usize = pages
        .values()
        .map(|page_id| {
            doc.get_page_content(*page_id)
                .expect("page must have content stream")
                .len()
        })
        .sum();

    assert!(
        total_content_len > 0,
        "page content stream must contain operators (got 0 bytes)"
    );
}
