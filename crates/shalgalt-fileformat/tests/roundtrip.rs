//! Write → read round-trip equality, for both plaintext and encrypted archives, plus the
//! streaming and determinism guarantees.

mod common;

use std::io::Read;

use shalgalt_fileformat::{open, open_manifest_only, write, FileFormatError, Manifest, ReadHandle};

#[test]
fn plaintext_roundtrip_preserves_every_entry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plain.shalgalt");

    let manifest = Manifest::new("Энгийн төсөл", common::fixed_time(), 4, false);
    write(&path, &manifest, common::sample_entries().into_iter(), None).unwrap();

    let (read_manifest, contents) = common::read_all(&path, None).unwrap();
    assert_eq!(read_manifest, manifest);
    assert_eq!(contents, common::expected_contents());
}

#[test]
fn encrypted_roundtrip_preserves_every_entry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secret.shalgalt");

    let manifest = Manifest::new("Шифрлэгдсэн төсөл", common::fixed_time(), 10, true);
    write(
        &path,
        &manifest,
        common::sample_entries().into_iter(),
        Some("correct horse battery staple"),
    )
    .unwrap();

    // (a) The manifest previews without a passphrase even though the archive is encrypted.
    let preview = open_manifest_only(&path).unwrap();
    assert_eq!(preview, manifest);
    assert!(preview.encrypted);

    // (b) Every payload entry decrypts to its original bytes with the right passphrase.
    let (read_manifest, contents) =
        common::read_all(&path, Some("correct horse battery staple")).unwrap();
    assert_eq!(read_manifest, manifest);
    assert_eq!(contents, common::expected_contents());
}

#[test]
fn large_entry_streams_from_path() {
    let dir = tempfile::tempdir().unwrap();
    let big_path = dir.path().join("exam.pdf");
    // 1 MiB of structured (non-trivially-compressible) bytes fed from disk via `from_path`.
    let big: Vec<u8> = (0..1024u32 * 1024)
        .map(|i| (i.wrapping_mul(2654435761) >> 16) as u8)
        .collect();
    std::fs::write(&big_path, &big).unwrap();

    let archive_path = dir.path().join("big.shalgalt");
    let manifest = Manifest::new("Том файл", common::fixed_time(), 1, false);
    let entries = vec![Ok((
        "exam.pdf".to_string(),
        ReadHandle::from_path(&big_path).unwrap(),
    ))];
    write(&archive_path, &manifest, entries.into_iter(), None).unwrap();

    let (_m, mut iter) = open(&archive_path, None).unwrap();
    assert_eq!(iter.entry_count(), 1);
    let (name, mut handle) = iter.next_entry().unwrap().unwrap();
    assert_eq!(name, "exam.pdf");
    let mut read_back = Vec::new();
    handle.read_to_end(&mut read_back).unwrap();
    // Drop the handle to release its borrow of the iterator before asking for the next entry.
    drop(handle);
    assert_eq!(read_back, big, "1 MiB entry must round-trip byte-for-byte");
    assert!(iter.next_entry().is_none());
}

/// True if `haystack` contains `needle` as a contiguous byte run.
fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn encrypted_archive_contains_no_payload_plaintext() {
    // Proves encryption is actually applied (not silently bypassed): a distinctive plaintext
    // marker is detectable in a plaintext archive but must NOT appear anywhere on disk in the
    // encrypted one. `.pdf` entries are `Stored` (not DEFLATEd), so the plaintext marker lands
    // verbatim — making the probe reliable.
    let dir = tempfile::tempdir().unwrap();
    let marker: &[u8] = b"UNIQUE-SECRET-MARKER-7f3a";

    let plain_path = dir.path().join("plain.shalgalt");
    let plain_manifest = Manifest::new("t", common::fixed_time(), 0, false);
    let plain_entries = vec![Ok((
        "secret.pdf".to_owned(),
        ReadHandle::from_bytes(marker.to_vec()),
    ))];
    write(
        &plain_path,
        &plain_manifest,
        plain_entries.into_iter(),
        None,
    )
    .unwrap();
    let plain_bytes = std::fs::read(&plain_path).unwrap();
    assert!(
        contains_subslice(&plain_bytes, marker),
        "marker must be detectable in a plaintext archive (probe sanity)"
    );

    let enc_path = dir.path().join("enc.shalgalt");
    let enc_manifest = Manifest::new("t", common::fixed_time(), 0, true);
    let enc_entries = vec![Ok((
        "secret.pdf".to_owned(),
        ReadHandle::from_bytes(marker.to_vec()),
    ))];
    write(
        &enc_path,
        &enc_manifest,
        enc_entries.into_iter(),
        Some("pw"),
    )
    .unwrap();
    let enc_bytes = std::fs::read(&enc_path).unwrap();
    assert!(
        !contains_subslice(&enc_bytes, marker),
        "payload plaintext must never appear in an encrypted archive"
    );
}

#[test]
fn manifest_only_archive_has_no_payload() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifest-only.shalgalt");
    let manifest = Manifest::new("Зөвхөн манифест", common::fixed_time(), 0, false);

    write(&path, &manifest, std::iter::empty(), None).unwrap();

    let (read_manifest, mut iter) = open(&path, None).unwrap();
    assert_eq!(read_manifest, manifest);
    assert_eq!(iter.entry_count(), 0);
    assert!(iter.next_entry().is_none());
    assert_eq!(open_manifest_only(&path).unwrap(), manifest);
}

#[test]
fn write_rejects_unsafe_entry_names() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Manifest::new("Нэр шалгах", common::fixed_time(), 0, false);

    for bad in [
        "",
        "manifest.json",
        "../escape",
        "/absolute",
        "a\\b",
        "a//b",
        "./relative",
        "a\0b",
    ] {
        let path = dir.path().join("x.shalgalt");
        let entries = vec![Ok((bad.to_string(), ReadHandle::from_bytes(b"x".to_vec())))];
        let err = write(&path, &manifest, entries.into_iter(), None).unwrap_err();
        assert_eq!(
            err.code(),
            "fileformat.malformed",
            "entry name {bad:?} must be rejected"
        );
        assert!(matches!(err, FileFormatError::Malformed(_)));
    }
}

#[test]
fn manifest_carries_no_pii_fields() {
    // The manifest is plaintext even in encrypted archives, so by contract it must never hold
    // student PII. The struct has no field that could, and a serialized instance proves it.
    let manifest = Manifest::new("Алгебр I — 12-р анги", common::fixed_time(), 30, true);
    let json = String::from_utf8(manifest.to_json_bytes().unwrap()).unwrap();
    let lower = json.to_lowercase();
    for forbidden in ["student", "roster", "grade", "score", "answer"] {
        assert!(
            !lower.contains(forbidden),
            "manifest JSON unexpectedly contains {forbidden:?}: {json}"
        );
    }
}

#[test]
fn entry_order_is_deterministic_and_manifest_excluded() {
    let dir = tempfile::tempdir().unwrap();
    let path_a = dir.path().join("a.shalgalt");
    let path_b = dir.path().join("b.shalgalt");
    let manifest = Manifest::new("Дараалал", common::fixed_time(), 4, false);

    write(
        &path_a,
        &manifest,
        common::sample_entries().into_iter(),
        None,
    )
    .unwrap();
    write(
        &path_b,
        &manifest,
        common::sample_entries().into_iter(),
        None,
    )
    .unwrap();

    let order_a = common::read_order(&path_a, None).unwrap();
    let order_b = common::read_order(&path_b, None).unwrap();

    let expected: Vec<String> = common::SAMPLE.iter().map(|(n, _)| n.to_string()).collect();
    assert_eq!(
        order_a, expected,
        "entries must yield in the order they were written"
    );
    assert_eq!(
        order_a, order_b,
        "two writes of the same input must agree on order"
    );
    assert!(
        !order_a.iter().any(|n| n == "manifest.json"),
        "manifest.json must never be yielded as a payload entry"
    );
}
