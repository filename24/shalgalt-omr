//! Passphrase-handling behavior on encrypted archives.

mod common;

use shalgalt_fileformat::{open, open_manifest_only, write, FileFormatError, Manifest};

/// Build a standard encrypted archive at `path` with the given passphrase.
fn write_encrypted(path: &std::path::Path, passphrase: &str) {
    let manifest = Manifest::new("Нууцлалтай", common::fixed_time(), 5, true);
    write(
        path,
        &manifest,
        common::sample_entries().into_iter(),
        Some(passphrase),
    )
    .unwrap();
}

#[test]
fn wrong_passphrase_returns_typed_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("enc.shalgalt");
    write_encrypted(&path, "the-real-passphrase");

    let (_manifest, mut iter) = open(&path, Some("a-different-passphrase")).unwrap();
    // The wrong passphrase surfaces when the first payload entry is requested.
    let err = match iter.next_entry() {
        Some(Err(e)) => e,
        Some(Ok((name, _))) => {
            panic!("expected an error, decrypted {name:?} with a wrong passphrase")
        }
        None => panic!("expected an entry, archive looked empty"),
    };
    assert_eq!(err.code(), "fileformat.bad_passphrase");
    assert!(matches!(err, FileFormatError::BadPassphrase));
}

#[test]
fn manifest_is_readable_without_a_passphrase() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("enc.shalgalt");
    write_encrypted(&path, "secret");

    // Both preview paths work with no passphrase, since the manifest is always plaintext.
    let preview = open_manifest_only(&path).unwrap();
    assert_eq!(preview.title, "Нууцлалтай");
    assert_eq!(preview.format_version, shalgalt_fileformat::FORMAT_VERSION);
    assert!(preview.encrypted);

    let (manifest, _iter) = open(&path, None).unwrap();
    assert_eq!(manifest, preview);
}

#[test]
fn missing_passphrase_on_encrypted_archive_is_bad_passphrase() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("enc.shalgalt");
    write_encrypted(&path, "secret");

    // Opening the manifest works, but extracting content without the key must fail clearly.
    let (_manifest, mut iter) = open(&path, None).unwrap();
    let err = match iter.next_entry() {
        Some(Err(e)) => e,
        Some(Ok((name, _))) => panic!("decrypted {name:?} without any passphrase"),
        None => panic!("expected an entry, archive looked empty"),
    };
    assert_eq!(err.code(), "fileformat.bad_passphrase");
    assert!(matches!(err, FileFormatError::BadPassphrase));
}
