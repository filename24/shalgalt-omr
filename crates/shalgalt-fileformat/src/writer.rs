//! Streaming writer for `.shalgalt` archives.
//!
//! [`write`] lays down `manifest.json` first as plaintext, then streams each payload entry —
//! age-encrypting and ASCII-armoring it on the fly when a passphrase is supplied. Entries are
//! piped through `io::copy`, so a multi-hundred-MB `exam.pdf` never lands in memory (Rule 1).

use std::fs::File;
use std::io::{self, BufWriter, Cursor};
use std::path::Path;

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod;

use crate::crypto;
use crate::error::{FileFormatError, FileFormatResult};
use crate::manifest::Manifest;
use crate::reader::ReadHandle;

/// Write a `.shalgalt` archive to `dest`.
///
/// `manifest.json` is written first and is **always plaintext**. Every entry from `entries` is
/// age-wrapped + ASCII-armored when `passphrase` is `Some`, and stored verbatim when it is
/// `None`. The caller supplies entries already in canonical order (`template.json`,
/// `answer-keys.json`, `metadata.json`, `students.csv`, `exam.pdf`); the manifest is prepended
/// automatically. `manifest.encrypted` MUST match `passphrase.is_some()` — this is enforced and
/// a mismatch returns [`FileFormatError::Malformed`], since the manifest is the preview surface
/// a reader trusts.
///
/// Each entry is stored with a 32-bit size field (no zip64), so a single entry must stay under
/// 4 GiB; a larger one fails with `fileformat.io`. That is far above any realistic exam project.
pub fn write(
    dest: impl AsRef<Path>,
    manifest: &Manifest,
    entries: impl Iterator<Item = FileFormatResult<(String, ReadHandle<'static>)>>,
    passphrase: Option<&str>,
) -> FileFormatResult<()> {
    // The manifest is the preview surface a reader trusts to decide whether a passphrase is
    // needed. If it disagreed with reality the archive would either advertise confidentiality
    // it does not have (encrypted flag, plaintext payload) or hand a reader raw ciphertext as
    // if it were data. Enforce the invariant rather than trusting the caller.
    if manifest.encrypted != passphrase.is_some() {
        return Err(FileFormatError::Malformed(
            "manifest.encrypted must match whether a passphrase was supplied".to_owned(),
        ));
    }

    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));

    // 1. manifest.json — plaintext, regardless of `passphrase`, so a reader can preview the
    //    project without the key. Small JSON ⇒ DEFLATE is worth it.
    let manifest_bytes = manifest.to_json_bytes()?;
    zip.start_file(
        crate::MANIFEST_ENTRY,
        file_options(CompressionMethod::Deflated),
    )?;
    io::copy(&mut Cursor::new(manifest_bytes), &mut zip)?;

    // 2. payload entries, streamed (and encrypted) one at a time.
    for entry in entries {
        let (name, mut handle) = entry?;
        validate_entry_name(&name)?;
        let method = pick_compression(&name, passphrase.is_some());
        zip.start_file(&name, file_options(method))?;

        match passphrase {
            None => {
                io::copy(&mut handle, &mut zip)?;
            }
            Some(pw) => {
                // `&mut zip` is the current entry's sink; the blanket `impl Write for &mut W`
                // lets it stand in for `W`. `finish_encrypt` returns that `&mut` borrow, which
                // we drop so the next `start_file` can finalize this entry.
                let mut enc = crypto::encrypting_writer(&mut zip, pw)?;
                io::copy(&mut handle, &mut enc)?;
                crypto::finish_encrypt(enc)?;
            }
        }
    }

    // 3. write the central directory. Dropping the writer without this loses the archive.
    zip.finish()?;
    Ok(())
}

/// Deterministic per-entry options: fixed compression method + a pinned modification time so
/// two writes of the same inputs produce byte-stable archives (no wall-clock leakage).
fn file_options(method: CompressionMethod) -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(method)
        .last_modified_time(zip::DateTime::default())
}

/// Encrypted entries (age ciphertext) and PDFs (already compressed) gain nothing from DEFLATE,
/// so they are stored. Small plaintext JSON / CSV is deflated.
fn pick_compression(name: &str, encrypted: bool) -> CompressionMethod {
    if encrypted || name.to_ascii_lowercase().ends_with(".pdf") {
        CompressionMethod::Stored
    } else {
        CompressionMethod::Deflated
    }
}

/// Reject entry names that could escape the extraction directory (zip-slip) or confuse a
/// cross-platform reader. Zip stores names verbatim and readers expect `/` separators.
fn validate_entry_name(name: &str) -> FileFormatResult<()> {
    // Reject exactly what the reader's `enclosed_name` sanitizer would refuse, so every name
    // that survives `write` round-trips to the identical name on read (write/read symmetry):
    // empty, the reserved manifest, backslashes, NUL, absolute paths, and `.`/`..`/empty
    // components.
    let unsafe_name = name.is_empty()
        || name == crate::MANIFEST_ENTRY
        || name.contains('\\')
        || name.contains('\0')
        || name.starts_with('/')
        || name
            .split('/')
            .any(|seg| seg == ".." || seg == "." || seg.is_empty());
    if unsafe_name {
        return Err(FileFormatError::Malformed(format!(
            "unsafe or reserved entry name: {name:?}"
        )));
    }
    Ok(())
}
