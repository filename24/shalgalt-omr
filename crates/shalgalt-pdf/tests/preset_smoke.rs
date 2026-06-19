//! Lightweight smoke test confirming the Mongolian-standard preset produces a valid PDF
//! with the expected group count. `golden.rs` owns deterministic byte-level comparison;
//! this file just checks the preset builder's shape (group count, marker count, single
//! page).

mod common;

use shalgalt_pdf::{render_template, PdfOptions};

use crate::common::preset;

#[test]
fn preset_builds_107_groups() {
    let tmpl = preset::mongolian_standard("Шалгалт");
    assert_eq!(
        tmpl.groups.len(),
        107,
        "4 cipher + 1 variant + 70 questions + 4×8 numerics = 107 (spec §6)"
    );
}

#[test]
fn preset_renders_to_valid_pdf() {
    let tmpl = preset::mongolian_standard("Шалгалт");
    let bytes = render_template(&tmpl, &PdfOptions::default()).expect("render");

    assert!(bytes.starts_with(b"%PDF-"));
    let doc = lopdf::Document::load_mem(&bytes).expect("parse");
    assert_eq!(doc.get_pages().len(), 1);
}
