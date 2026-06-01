//! Streaming reader for `.shalgalt` archives.
//!
//! [`open`] parses and version-checks the (always-plaintext) `manifest.json`, then hands back
//! an [`EntryIter`] that streams the remaining entries one at a time — decrypting on demand
//! when a passphrase is supplied. Nothing but the tiny manifest is ever buffered whole
//! (Rule 1).

use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::Path;

use age::secrecy::ExposeSecret;

use crate::crypto;
use crate::error::{FileFormatError, FileFormatResult};
use crate::manifest::Manifest;

/// Upper bound on the buffered `manifest.json`. The manifest is the only entry read whole; a
/// hostile archive could otherwise declare a multi-GB "manifest" and OOM the process. Any real
/// manifest is a few hundred bytes, so 1 MiB is enormous headroom.
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

/// A streaming `Read` over one archive entry.
///
/// Backed by a boxed trait object so the same handle can carry a borrowed zip entry, an
/// on-the-fly age decrypt stream, an owned file (`from_path`), or owned bytes (`from_bytes`)
/// without leaking which one to the caller. The lifetime `'a` ties a borrowed handle to the
/// [`EntryIter`] it came from; owned handles are `'static`.
pub struct ReadHandle<'a>(Box<dyn Read + 'a>);

impl<'a> ReadHandle<'a> {
    /// Wrap any reader as a handle borrowing for `'a`. Used internally by [`EntryIter`].
    pub(crate) fn from_reader(r: impl Read + 'a) -> Self {
        ReadHandle(Box::new(r))
    }
}

impl ReadHandle<'static> {
    /// Stream a file from disk. This is how callers feed large payloads (e.g. `exam.pdf`) into
    /// [`crate::write`] without buffering them in memory.
    pub fn from_path(path: impl AsRef<Path>) -> FileFormatResult<ReadHandle<'static>> {
        let file = File::open(path)?;
        Ok(ReadHandle(Box::new(BufReader::new(file))))
    }

    /// Wrap small in-memory bytes (e.g. a serialized `template.json`, a few KB) as a handle.
    pub fn from_bytes(bytes: Vec<u8>) -> ReadHandle<'static> {
        ReadHandle(Box::new(Cursor::new(bytes)))
    }
}

impl Read for ReadHandle<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

/// Streams the non-manifest entries of an open archive, one at a time.
///
/// Owns the underlying [`zip::ZipArchive`] and yields entries via [`EntryIter::next_entry`].
/// It deliberately does **not** implement [`Iterator`]: each yielded [`ReadHandle`] borrows the
/// archive, which `Iterator::Item` cannot express. The borrow checker therefore forces the
/// caller to consume (and drop) each handle before requesting the next entry.
///
/// The payload entry list (`(zip_index, name)`, manifest excluded) is resolved up front from
/// the zip central directory — cheap metadata, no payload bytes — so `next_entry` performs a
/// single `by_index` per call and never re-borrows the archive inside a loop.
pub struct EntryIter {
    archive: zip::ZipArchive<BufReader<File>>,
    entries: Vec<(usize, String)>,
    cursor: usize,
    /// Held as a zeroizing `SecretString` (mirroring `crypto.rs`) so the iterator's copy of the
    /// passphrase is wiped on drop rather than lingering in a plain `String`.
    passphrase: Option<age::secrecy::SecretString>,
    encrypted: bool,
}

impl EntryIter {
    /// Yield the next `(entry_name, handle)` (manifest already excluded). Returns `None` when
    /// every payload entry has been visited.
    ///
    /// - Plaintext archive: the handle streams the raw entry bytes.
    /// - Encrypted archive **with** a passphrase: the handle streams decrypted plaintext; a
    ///   wrong passphrase surfaces here as [`FileFormatError::BadPassphrase`].
    /// - Encrypted archive **without** a passphrase: returns [`FileFormatError::BadPassphrase`]
    ///   — the manifest is still readable via [`open_manifest_only`], but content needs the key.
    pub fn next_entry(&mut self) -> Option<FileFormatResult<(String, ReadHandle<'_>)>> {
        if self.cursor >= self.entries.len() {
            return None;
        }
        let (idx, name) = self.entries[self.cursor].clone();
        self.cursor += 1;

        // Content cannot be extracted from an encrypted archive without the key.
        if self.encrypted && self.passphrase.is_none() {
            return Some(Err(FileFormatError::BadPassphrase));
        }

        let entry = match self.archive.by_index(idx) {
            Ok(entry) => entry,
            Err(e) => return Some(Err(e.into())),
        };

        if self.encrypted {
            // Guarded above: when encrypted, a passphrase is present.
            let pw = self
                .passphrase
                .as_ref()
                .map(|s| s.expose_secret())
                .unwrap_or_default();
            return Some(match crypto::decrypting_reader(entry, pw) {
                Ok(reader) => Ok((name, ReadHandle::from_reader(reader))),
                Err(e) => Err(e),
            });
        }

        Some(Ok((name, ReadHandle::from_reader(entry))))
    }

    /// Number of payload entries (everything except `manifest.json`).
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Open a `.shalgalt` archive: parse + version-check `manifest.json`, then return it alongside
/// a streaming iterator over the remaining entries.
///
/// `passphrase` is `None` for a plaintext archive (or when only the manifest is needed). The
/// version gate runs before any decryption, so a future-format file fails fast with
/// [`FileFormatError::VersionTooNew`].
pub fn open(
    path: impl AsRef<Path>,
    passphrase: Option<&str>,
) -> FileFormatResult<(Manifest, EntryIter)> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;
    let manifest = read_manifest(&mut archive)?;
    let entries = collect_payload_entries(&mut archive)?;
    let iter = EntryIter {
        archive,
        entries,
        cursor: 0,
        passphrase: passphrase.map(age::secrecy::SecretString::from),
        encrypted: manifest.encrypted,
    };
    Ok((manifest, iter))
}

/// Cheap preview: read and version-check only the manifest, never building the entry iterator.
/// Always succeeds without a passphrase, even on an encrypted archive (the manifest is plaintext).
pub fn open_manifest_only(path: impl AsRef<Path>) -> FileFormatResult<Manifest> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;
    read_manifest(&mut archive)
}

/// Read `manifest.json` (the only entry small enough to buffer whole) and run the version gate.
fn read_manifest<R: Read + Seek>(archive: &mut zip::ZipArchive<R>) -> FileFormatResult<Manifest> {
    let mut entry = match archive.by_name(crate::MANIFEST_ENTRY) {
        Ok(entry) => entry,
        Err(zip::result::ZipError::FileNotFound) => {
            return Err(FileFormatError::Malformed(
                "archive is missing manifest.json — not a valid .shalgalt file".to_owned(),
            ))
        }
        Err(other) => return Err(other.into()),
    };
    // Cap the read so a hostile "manifest.json" cannot exhaust memory. Reading one extra byte
    // lets us detect (rather than silently truncate) an over-large manifest.
    let mut buf = Vec::new();
    let read = entry
        .by_ref()
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut buf)?;
    if read as u64 > MAX_MANIFEST_BYTES {
        return Err(FileFormatError::Malformed(
            "manifest.json exceeds the maximum allowed size".to_owned(),
        ));
    }
    Manifest::from_reader(&buf[..])
}

/// Resolve the ordered list of payload entries `(zip_index, sanitized_name)` from the central
/// directory, excluding `manifest.json`. Reads only metadata — never decompresses payload —
/// and rejects unsafe (zip-slip) names up front so iteration cannot yield one.
fn collect_payload_entries<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> FileFormatResult<Vec<(usize, String)>> {
    let mut entries = Vec::with_capacity(archive.len().saturating_sub(1));
    let mut seen = std::collections::BTreeSet::new();
    for idx in 0..archive.len() {
        let entry = archive.by_index(idx)?;
        let name = match entry.enclosed_name() {
            Some(path) => path.to_string_lossy().into_owned(),
            None => {
                return Err(FileFormatError::Malformed(format!(
                    "unsafe entry name at index {idx}"
                )))
            }
        };
        drop(entry);
        // Reject duplicate names so a hostile archive cannot shadow one entry with another.
        if !seen.insert(name.clone()) {
            return Err(FileFormatError::Malformed(format!(
                "duplicate entry name: {name:?}"
            )));
        }
        if name != crate::MANIFEST_ENTRY {
            entries.push((idx, name));
        }
    }
    Ok(entries)
}
