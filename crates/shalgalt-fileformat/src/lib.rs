//! `.shalgalt` project file: a zip container with optional `age` passphrase encryption.
//!
//! A `.shalgalt` file bundles an entire exam project — template, answer keys, metadata,
//! optional roster, optional original PDF — into a single shareable archive. `manifest.json`
//! is **always plaintext** (so the desktop app can preview title / date / sheet count without
//! a passphrase); every other entry is `age`-wrapped and ASCII-armored when a passphrase is
//! supplied.
//!
//! ## Rule 1 — streaming, never buffer the whole archive
//!
//! A project can carry hundreds of MB of scanned PDFs. Both [`write`] and [`open`] stream
//! one entry at a time through [`ReadHandle`]; the only bytes ever held whole in memory are
//! the tiny `manifest.json`. See `crates/shalgalt-fileformat/AGENTS.md`.
//!
//! ## Public surface
//!
//! - [`write`] — create an archive from a manifest + an iterator of named entries.
//! - [`open`] — parse + version-check the manifest, then stream the remaining entries via
//!   [`EntryIter::next_entry`].
//! - [`open_manifest_only`] — cheap preview: read just the (always-plaintext) manifest.
//! - [`Manifest`], [`ReadHandle`], [`FileFormatError`].
//!
//! There is intentionally no `ProjectFile` facade (YAGNI): the free functions plus
//! [`ReadHandle`] cover every P4 caller, and the Tauri command layer (P4-03) builds its own
//! domain helpers on top. A facade can be added later without breaking these signatures.

pub mod error;
pub mod manifest;
pub mod reader;
pub mod writer;

pub(crate) mod crypto;

pub use error::{FileFormatError, FileFormatResult};
pub use manifest::Manifest;
pub use reader::{open, open_manifest_only, EntryIter, ReadHandle};
pub use writer::write;

/// The `format_version` integer this build understands.
///
/// A loader refuses any manifest whose `format_version` is greater than this and returns
/// [`FileFormatError::VersionTooNew`]. Bump this only on a breaking change to the archive
/// layout or the manifest schema.
pub const FORMAT_VERSION: u32 = 1;

/// The canonical name of the always-plaintext manifest entry at the archive root.
pub const MANIFEST_ENTRY: &str = "manifest.json";
