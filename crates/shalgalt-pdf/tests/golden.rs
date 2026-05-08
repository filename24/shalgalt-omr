//! Golden-file regression test.
//!
//! Renders the Mongolian-standard preset deterministically and byte-compares against
//! `tests/golden_data/mongolian_standard.pdf`. The PDF trailer's `/ID` array and the
//! `/CreationDate` / `/ModDate` metadata fields can vary between runs, so we normalize
//! those before comparison.
//!
//! To intentionally accept a new golden:
//!
//! ```bash
//! cargo test -p shalgalt-pdf --test golden -- --ignored regenerate_golden
//! ```

mod common;

use std::path::PathBuf;

use shalgalt_pdf::{render_template, BubbleStyle, HeaderText, PdfOptions};

use crate::common::preset;

const GOLDEN_PATH: &str = "tests/golden_data/mongolian_standard.pdf";

/// Deterministic render options — every timestamp pinned to the unix epoch.
fn deterministic_options() -> PdfOptions {
    PdfOptions {
        header: HeaderText {
            title: Some("Шалгалтын хариултын хуудас".to_string()),
            subtitle: Some("Жишээ загвар".to_string()),
            school: Some("Сургууль".to_string()),
            teacher: Some("Багш".to_string()),
        },
        instructions: Some("Зөв хариултыг бөглөнө үү".to_string()),
        bubble_style: BubbleStyle::default(),
        created_at: Some(printpdf::DateTime::epoch()),
        ..PdfOptions::default()
    }
}

fn normalize_pdf(bytes: &[u8]) -> Vec<u8> {
    // Replace the `/ID [<...> <...>]` line with an empty array. A plain ASCII byte search
    // is sufficient — the PDF trailer is emitted as text.
    let mut out = bytes.to_vec();

    if let Some(start) = find_subsequence(&out, b"/ID [") {
        if let Some(end_rel) = find_subsequence(&out[start..], b"]") {
            let end = start + end_rel + 1;
            out.splice(start..end, b"/ID []".iter().copied());
        }
    }

    // /CreationDate / /ModDate are stable when the caller pinned them to the epoch; we
    // could normalize here too, but the deterministic options already guarantee stability.
    out
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[test]
fn golden_file_matches() {
    let template = preset::mongolian_standard("Жишээ загвар");
    let bytes = render_template(&template, &deterministic_options())
        .expect("render Mongolian-standard preset");
    let normalized = normalize_pdf(&bytes);

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_PATH);
    let expected = std::fs::read(&path).unwrap_or_else(|err| {
        panic!(
            "missing golden file at {} ({err}). Run:\n  \
             cargo test -p shalgalt-pdf --test golden -- --ignored regenerate_golden",
            path.display()
        );
    });

    if normalized != expected {
        // Wrote the actual bytes to a sibling .actual file so the diff is inspectable.
        let actual_path = path.with_extension("pdf.actual");
        let _ = std::fs::write(&actual_path, &normalized);
        panic!(
            "golden file mismatch (len: actual={}, golden={}). Wrote actual bytes to {}.\n\
             To accept the new layout intentionally, run:\n  \
             cargo test -p shalgalt-pdf --test golden -- --ignored regenerate_golden",
            normalized.len(),
            expected.len(),
            actual_path.display()
        );
    }
}

#[test]
#[ignore = "regeneration is opt-in — pass --ignored to enable"]
fn regenerate_golden() {
    let template = preset::mongolian_standard("Жишээ загвар");
    let bytes = render_template(&template, &deterministic_options())
        .expect("render Mongolian-standard preset");
    let normalized = normalize_pdf(&bytes);

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create golden dir");
    }
    std::fs::write(&path, &normalized).expect("write golden file");
    eprintln!("wrote {} ({} bytes)", path.display(), normalized.len());
}
