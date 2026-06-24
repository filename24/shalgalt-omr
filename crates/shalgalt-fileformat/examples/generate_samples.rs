//! Generate the three documentation sample `.shalgalt` projects (P7-04).
//!
//! Reads the plaintext payload assets from a payload directory and packages them into three
//! archives that the user manual walks through:
//!
//! * `sample-open.shalgalt`      — unencrypted, no PDF (smallest, opens in any zip viewer).
//! * `sample-with-pdf.shalgalt`  — unencrypted, includes `exam.pdf`.
//! * `sample-encrypted.shalgalt` — age-passphrase encrypted, includes `exam.pdf`.
//!
//! The encrypted sample's passphrase is intentionally public so the manual can demonstrate
//! the open-with-passphrase flow:  **`shalgalt-2026`**.
//!
//! Usage:
//!   cargo run -p shalgalt-fileformat --example generate_samples
//!   cargo run -p shalgalt-fileformat --example generate_samples -- <payload_dir> <out_dir>

use std::error::Error;
use std::path::{Path, PathBuf};

use shalgalt_fileformat::{write, Manifest, ReadHandle};

/// Public, documented passphrase for the encrypted sample.
const SAMPLE_PASSPHRASE: &str = "shalgalt-2026";

/// Canonical entry order (writer contract). `exam.pdf` is appended only for the PDF samples.
const BASE_ENTRIES: &[&str] = &[
    "template.json",
    "answer-keys.json",
    "metadata.json",
    "students.csv",
];

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let payload_dir = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "docs/user/samples/_payload".to_owned()),
    );
    let out_dir = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "docs/user/samples".to_owned()),
    );
    std::fs::create_dir_all(&out_dir)?;

    // Fixed timestamp so regenerating produces byte-identical archives (diff-friendly).
    let created_at =
        chrono::DateTime::parse_from_rfc3339("2026-05-20T09:00:00Z")?.with_timezone(&chrono::Utc);
    let title = "Жишээ шалгалт — 5 асуулт (A–D)";
    let sheet_count: u32 = 5;

    // 1. Open project, no PDF.
    build(
        &out_dir.join("sample-open.shalgalt"),
        Manifest::new(title, created_at, sheet_count, false),
        &payload_dir,
        BASE_ENTRIES,
        None,
    )?;

    // 2. Open project with the rendered exam PDF.
    build(
        &out_dir.join("sample-with-pdf.shalgalt"),
        Manifest::new(title, created_at, sheet_count, false),
        &payload_dir,
        &with_pdf(),
        None,
    )?;

    // 3. Encrypted project with PDF. The manifest stays plaintext and carries a hint.
    let mut encrypted_manifest = Manifest::new(title, created_at, sheet_count, true);
    encrypted_manifest.hint = Some("Spring 2026 — passphrase: shalgalt-2026".to_owned());
    build(
        &out_dir.join("sample-encrypted.shalgalt"),
        encrypted_manifest,
        &payload_dir,
        &with_pdf(),
        Some(SAMPLE_PASSPHRASE),
    )?;

    eprintln!("Generated 3 sample archives in {}", out_dir.display());
    Ok(())
}

fn with_pdf() -> Vec<&'static str> {
    let mut entries = BASE_ENTRIES.to_vec();
    entries.push("exam.pdf");
    entries
}

/// Stream the named payload files into a single `.shalgalt` archive.
fn build(
    dest: &Path,
    manifest: Manifest,
    payload_dir: &Path,
    entry_names: &[&str],
    passphrase: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let names: Vec<String> = entry_names.iter().map(|s| s.to_string()).collect();
    let dir = payload_dir.to_path_buf();

    let entries = names.into_iter().map(move |name| {
        let handle = ReadHandle::from_path(dir.join(&name))?;
        Ok((name, handle))
    });

    write(dest, &manifest, entries, passphrase)?;
    eprintln!("  wrote {}", dest.display());
    Ok(())
}
