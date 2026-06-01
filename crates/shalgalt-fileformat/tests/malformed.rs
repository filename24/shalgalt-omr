//! Hostile / malformed input handling — archives that `write()` cannot itself produce, built
//! directly with the `zip` crate, plus the writer's consistency guard. Every case must yield a
//! typed [`FileFormatError`] (never a panic, never silent garbage-as-success).

mod common;

use std::io::{Read, Write};
use std::path::Path;

use shalgalt_fileformat::{open, open_manifest_only, write, Manifest};

const VALID_MANIFEST_JSON: &[u8] =
    br#"{"format_version":1,"title":"t","created_at":"2026-05-31T09:00:00Z","sheet_count":0,"encrypted":false}"#;

/// Hand-build a raw zip with the given entries (no manifest logic, no encryption) so tests can
/// forge structures `write()` would refuse to create.
fn build_raw_zip(path: &Path, entries: &[(&str, &[u8])]) {
    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in entries {
        zip.start_file(*name, opts).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn reads_externally_built_plaintext_archive() {
    // Interop: a `.shalgalt` assembled by an external zip tool (manifest + plaintext entries,
    // no help from our own `write`) must read back through `open`.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("external.shalgalt");
    build_raw_zip(
        &path,
        &[
            ("manifest.json", VALID_MANIFEST_JSON),
            ("template.json", br#"{"k":1}"#),
            ("students.csv", b"id,name\n1,Bat\n"),
        ],
    );

    let manifest = open_manifest_only(&path).unwrap();
    assert_eq!(manifest.title, "t");
    assert!(!manifest.encrypted);

    let (_m, mut iter) = open(&path, None).unwrap();
    let mut names = Vec::new();
    while let Some(res) = iter.next_entry() {
        let (name, mut handle) = res.unwrap();
        let mut buf = Vec::new();
        handle.read_to_end(&mut buf).unwrap();
        drop(handle);
        names.push(name);
    }
    assert_eq!(
        names,
        vec!["template.json".to_owned(), "students.csv".to_owned()]
    );
}

#[test]
fn non_zip_input_is_malformed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("garbage.shalgalt");
    std::fs::write(&path, b"this is plainly not a zip archive, just some text").unwrap();

    assert_eq!(
        open_manifest_only(&path).unwrap_err().code(),
        "fileformat.malformed"
    );
    assert_eq!(
        open(&path, None).map(|_| ()).unwrap_err().code(),
        "fileformat.malformed"
    );
}

#[test]
fn zip_without_manifest_is_malformed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("no-manifest.shalgalt");
    build_raw_zip(&path, &[("data.bin", b"hello, no manifest here")]);

    assert_eq!(
        open_manifest_only(&path).unwrap_err().code(),
        "fileformat.malformed"
    );
    assert_eq!(
        open(&path, None).map(|_| ()).unwrap_err().code(),
        "fileformat.malformed"
    );
}

#[test]
fn garbled_manifest_json_is_serde_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("garbled.shalgalt");
    build_raw_zip(&path, &[("manifest.json", b"{ not valid json")]);

    assert_eq!(
        open_manifest_only(&path).unwrap_err().code(),
        "fileformat.serde"
    );
    assert_eq!(
        open(&path, None).map(|_| ()).unwrap_err().code(),
        "fileformat.serde"
    );
}

// NOTE: the reader rejects duplicate entry names (defense-in-depth in `collect_payload_entries`)
// against hand-crafted archives, but there is no test for it: the `zip` writer refuses to emit a
// duplicate name (`InvalidArchive("Duplicate filename")`), so the malicious input cannot be built
// through the safe API and crafting it at the byte level is out of scope here.

#[test]
fn encrypted_flag_with_plaintext_entry_errors_not_panics() {
    // A forged archive: manifest claims encryption, but a payload entry is raw plaintext (not
    // age). Decrypting non-age bytes must fail cleanly, never panic and never return garbage.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("forged.shalgalt");
    let manifest_json = br#"{"format_version":1,"title":"t","created_at":"2026-05-31T09:00:00Z","sheet_count":0,"encrypted":true}"#;
    build_raw_zip(
        &path,
        &[("manifest.json", manifest_json), ("data.bin", b"plaintext")],
    );

    let (manifest, mut iter) = open(&path, Some("any-passphrase")).unwrap();
    assert!(manifest.encrypted);
    let result = iter.next_entry().expect("one payload entry");
    assert!(
        result.is_err(),
        "decrypting non-age bytes must surface an error"
    );
}

#[test]
fn write_rejects_encrypted_flag_passphrase_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mismatch.shalgalt");

    // (a) manifest declares encryption but no passphrase is supplied — the dangerous case (a
    //     false confidentiality label over plaintext payload). Must be refused.
    let encrypted_manifest = Manifest::new("t", common::fixed_time(), 0, true);
    let err = write(
        &path,
        &encrypted_manifest,
        common::sample_entries().into_iter(),
        None,
    )
    .unwrap_err();
    assert_eq!(err.code(), "fileformat.malformed");

    // (b) manifest declares plaintext but a passphrase is supplied — entries would be encrypted
    //     yet the reader would hand back raw ciphertext. Must be refused.
    let plain_manifest = Manifest::new("t", common::fixed_time(), 0, false);
    let err = write(
        &path,
        &plain_manifest,
        common::sample_entries().into_iter(),
        Some("pw"),
    )
    .unwrap_err();
    assert_eq!(err.code(), "fileformat.malformed");
}
