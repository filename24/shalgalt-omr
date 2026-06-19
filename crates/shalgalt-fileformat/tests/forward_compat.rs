//! Forward-compatibility: a loader must refuse a `format_version` it does not understand, and
//! must do so *before* attempting any decryption.

mod common;

use shalgalt_fileformat::{
    open, open_manifest_only, write, FileFormatError, Manifest, FORMAT_VERSION,
};

/// Write an archive whose manifest claims a future `format_version`. The writer does not police
/// the version field, so mutating it on a normal `Manifest` is enough to forge a future file.
fn write_with_version(
    path: &std::path::Path,
    format_version: u32,
    encrypted: bool,
    passphrase: Option<&str>,
) {
    let mut manifest = Manifest::new("Ирээдүйн хувилбар", common::fixed_time(), 1, encrypted);
    manifest.format_version = format_version;
    write(
        path,
        &manifest,
        common::sample_entries().into_iter(),
        passphrase,
    )
    .unwrap();
}

#[test]
fn future_version_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("future.shalgalt");
    write_with_version(&path, 999, false, None);

    let err = open_manifest_only(&path).unwrap_err();
    assert_eq!(err.code(), "fileformat.version_too_new");
    match err {
        FileFormatError::VersionTooNew { found, supported } => {
            assert_eq!(found, 999);
            assert_eq!(supported, FORMAT_VERSION);
        }
        other => panic!("expected VersionTooNew, got {other:?}"),
    }

    // `open` must reject it too, at the manifest stage.
    let open_err = open(&path, None).map(|_| ()).unwrap_err();
    assert_eq!(open_err.code(), "fileformat.version_too_new");
}

#[test]
fn current_version_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("current.shalgalt");
    write_with_version(&path, FORMAT_VERSION, false, None);

    let manifest = open_manifest_only(&path).unwrap();
    assert_eq!(manifest.format_version, FORMAT_VERSION);
}

#[test]
fn future_version_refused_before_any_decryption() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("future-encrypted.shalgalt");
    // Encrypted archive whose *plaintext* manifest carries a future version.
    write_with_version(&path, 999, true, Some("secret"));

    // Even with a deliberately wrong passphrase, the version gate fires first — the error is
    // VersionTooNew, never BadPassphrase, proving the version check precedes decryption.
    let err = open(&path, Some("wrong-passphrase"))
        .map(|_| ())
        .unwrap_err();
    assert_eq!(err.code(), "fileformat.version_too_new");
    assert!(matches!(
        err,
        FileFormatError::VersionTooNew { found: 999, .. }
    ));
}
