//! Shared helpers for the `.shalgalt` integration tests.
//!
//! Integration tests are a separate crate, so they reach the library only through its public
//! API plus the dev-dependencies (`tempfile`, `chrono`).

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use shalgalt_fileformat::{open, FileFormatResult, Manifest, ReadHandle};

/// The canonical payload set a `.shalgalt` carries (manifest is added by `write`). Both the
/// entry iterator fed to `write` and the expected-contents map derive from this single source.
pub const SAMPLE: &[(&str, &[u8])] = &[
    (
        "template.json",
        r#"{"template":true,"version":1}"#.as_bytes(),
    ),
    (
        "answer-keys.json",
        r#"{"A":[1,2,3],"B":[4,5,6]}"#.as_bytes(),
    ),
    // Cyrillic content exercises UTF-8 payloads through the streaming path.
    (
        "metadata.json",
        r#"{"school":"Сургууль №1","subject":"Алгебр"}"#.as_bytes(),
    ),
    ("students.csv", "id,name\n1,Bat\n2,Saraa\n".as_bytes()),
];

/// A fixed timestamp so manifests are deterministic across runs.
pub fn fixed_time() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339("2026-05-31T09:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc)
}

/// Build the [`SAMPLE`] entries as in-memory handles, in canonical order.
pub fn sample_entries() -> Vec<FileFormatResult<(String, ReadHandle<'static>)>> {
    SAMPLE
        .iter()
        .map(|(name, bytes)| Ok((name.to_string(), ReadHandle::from_bytes(bytes.to_vec()))))
        .collect()
}

/// The expected `name -> bytes` map after a round trip of [`SAMPLE`].
pub fn expected_contents() -> BTreeMap<String, Vec<u8>> {
    SAMPLE
        .iter()
        .map(|(name, bytes)| (name.to_string(), bytes.to_vec()))
        .collect()
}

/// Extract every payload entry into a `name -> bytes` map, streaming each handle to completion.
pub fn read_all(
    path: &Path,
    passphrase: Option<&str>,
) -> FileFormatResult<(Manifest, BTreeMap<String, Vec<u8>>)> {
    let (manifest, mut iter) = open(path, passphrase)?;
    let mut map = BTreeMap::new();
    while let Some(res) = iter.next_entry() {
        let (name, mut handle) = res?;
        let mut buf = Vec::new();
        handle.read_to_end(&mut buf)?;
        map.insert(name, buf);
    }
    Ok((manifest, map))
}

/// Collect payload entry names in the order `next_entry` yields them (handles left unread).
pub fn read_order(path: &Path, passphrase: Option<&str>) -> FileFormatResult<Vec<String>> {
    let (_manifest, mut iter) = open(path, passphrase)?;
    let mut names = Vec::new();
    while let Some(res) = iter.next_entry() {
        let (name, _handle) = res?;
        names.push(name);
    }
    Ok(names)
}
