//! Regression test for issue #75 — verifies the renderer's text-op shape.
//!
//! After the follow-up redesign, every bubble carries its choice/digit label INSIDE the
//! circle, plus one row label per group, plus header / sidebar text. The guardrails in
//! this file pin the output shape so we don't drift back into a layout that silently
//! drops or duplicates labels.

mod common;

use printpdf::{Op, ParsedFont, PdfDocument};
use shalgalt_pdf::{build_page_ops_for_test, BubbleStyle, HeaderText, PdfOptions};

use crate::common::preset;

const NOTO_SANS: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
const NOTO_SANS_MONGOLIAN: &[u8] = include_bytes!("../assets/fonts/NotoSansMongolian-Regular.ttf");

fn build_test_ops(opts: &PdfOptions) -> Vec<Op> {
    let template = preset::mongolian_standard("Жишээ загвар");

    let mut warnings = Vec::new();
    let body_font = ParsedFont::from_bytes(NOTO_SANS, 0, &mut warnings).expect("parse NotoSans");
    let mut sidebar_warnings = Vec::new();
    let mongolian_font = ParsedFont::from_bytes(NOTO_SANS_MONGOLIAN, 0, &mut sidebar_warnings)
        .expect("parse NotoSans Mongolian");

    let mut doc = PdfDocument::new("regression-test");
    let body_font_id = doc.add_font(&body_font);
    let sidebar_font_id = doc.add_font(&mongolian_font);

    build_page_ops_for_test(&template, opts, &body_font_id, Some(&sidebar_font_id))
}

fn count_show_text(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::ShowText { .. }))
        .count()
}

fn total_bubble_count() -> usize {
    let template = preset::mongolian_standard("Жишээ загвар");
    template.groups.iter().map(|g| g.bubbles.len()).sum()
}

fn group_count_with_labels() -> usize {
    let template = preset::mongolian_standard("Жишээ загвар");
    template
        .groups
        .iter()
        .filter(|g| !g.label.trim().is_empty())
        .count()
}

#[test]
fn every_bubble_has_a_label_inside() {
    let opts = PdfOptions {
        header: HeaderText::default(),
        instructions: None,
        bubble_style: BubbleStyle::default(),
        manual_id_slots: false,
        ..PdfOptions::default()
    };

    let ops = build_test_ops(&opts);
    let text_ops = count_show_text(&ops);
    let bubbles = total_bubble_count();
    let row_labels = group_count_with_labels();

    // Lower bound: at least one ShowText per bubble (label inside) plus one per labelled
    // group (row label). Header / variant / sidebar are off in this test.
    let expected_min = bubbles + row_labels;
    assert!(
        text_ops >= expected_min,
        "expected at least {expected_min} text ops (bubbles={bubbles} + row labels={row_labels}), got {text_ops}"
    );
}

#[test]
fn text_op_count_is_bounded_above() {
    let opts = PdfOptions {
        header: HeaderText {
            title: Some("Жишээ загвар".to_string()),
            subtitle: Some("Жишээ".to_string()),
            school: Some("Сургууль".to_string()),
            teacher: Some("Багш".to_string()),
        },
        instructions: Some("Зөв хариултыг бөглөнө үү".to_string()),
        bubble_style: BubbleStyle::default(),
        manual_id_slots: true,
        ..PdfOptions::default()
    };

    let ops = build_test_ops(&opts);
    let text_ops = count_show_text(&ops);

    // Upper bound: each bubble emits exactly 1 label; each labelled group adds at most
    // 1 row label; header is at most 4 lines; sidebar is rendered as one ShowText per
    // codepoint due to printpdf rotation handling, so allow generous headroom.
    let bubbles = total_bubble_count();
    let row_labels = group_count_with_labels();
    let upper_bound = bubbles + row_labels + 4 + 64;
    assert!(
        text_ops <= upper_bound,
        "text_ops ({text_ops}) exceeded upper bound ({upper_bound})"
    );
}

#[test]
fn manual_id_slots_render_only_for_student_id_rows() {
    let with_slots = PdfOptions {
        manual_id_slots: true,
        ..PdfOptions::default()
    };
    let without_slots = PdfOptions {
        manual_id_slots: false,
        ..PdfOptions::default()
    };

    let ops_with = build_test_ops(&with_slots);
    let ops_without = build_test_ops(&without_slots);

    let line_count = |ops: &[Op]| {
        ops.iter()
            .filter(|o| matches!(o, Op::DrawLine { .. }))
            .count()
    };

    let template = preset::mongolian_standard("Жишээ загвар");
    let student_id_rows = template
        .groups
        .iter()
        .filter(|g| matches!(g.kind, shalgalt_pdf::domain::BubbleKind::StudentId))
        .count();

    assert_eq!(
        line_count(&ops_with) - line_count(&ops_without),
        student_id_rows,
        "manual_id_slots=true should add exactly one underline per student-id group"
    );
}
