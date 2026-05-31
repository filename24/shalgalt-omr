//! Crate error type.
//!
//! Mirrors the `AppError` pattern in `shalgalt-core` but exposes a **public** `code()` so
//! the desktop command layer (P4-03) can map a stable English code to a Mongolian string
//! exactly the way it already maps `AppError::code`.
//!
//! SECURITY: never forward the inner `age` error text to the user or to `tracing!`. An age
//! error may carry recipient / passphrase-hint metadata. [`FileFormatError::BadPassphrase`]
//! is intentionally fieldless so a wrong-passphrase attempt cannot leak crypto internals.

/// Error returned by every public entry point in this crate.
///
/// The set of strings returned by [`FileFormatError::code`] is **locked** — the desktop
/// string-table keys to them, so renaming one is a breaking change.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FileFormatError {
    /// `manifest.format_version` is greater than [`crate::FORMAT_VERSION`]. Checked right
    /// after `manifest.json` is parsed and **before** any decryption or entry iteration, so
    /// an unreadable future archive fails fast with a precise reason rather than a crypto
    /// or zip error further down.
    #[error("unsupported format version {found} (this build supports up to {supported})")]
    VersionTooNew { found: u32, supported: u32 },

    /// Wrong passphrase (age 0.11 surfaces this as `DecryptError::DecryptionFailed` for scrypt
    /// files; `KeyDecryptionFailed` / `NoMatchingKeys` are mapped here too), or a passphrase was
    /// required — the archive's `manifest.encrypted` is `true` — but none was supplied. Fieldless
    /// on purpose so it never leaks age internals.
    #[error("authentication failed: wrong or missing passphrase")]
    BadPassphrase,

    /// Structurally invalid archive: a missing or garbled `manifest.json`, an invalid zip, an
    /// unsafe entry name (zip-slip), or an age structural failure that is *not* a wrong
    /// passphrase.
    #[error("malformed archive: {0}")]
    Malformed(String),

    /// Underlying `age` crypto failure that is *not* a wrong passphrase (e.g. excessive scrypt
    /// work factor, or an encrypt-side I/O failure). The string is a sanitized summary; the
    /// raw age message is never surfaced to the user.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// Filesystem / I/O failure (open, seek, copy, flush).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON (de)serialization failure of the manifest.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl FileFormatError {
    /// Stable, English, machine-readable code. The desktop layer maps this to a Mongolian
    /// string. These strings are **locked** — changing one is a breaking change.
    ///
    /// The `Crypto` carrier variant folds into `fileformat.malformed` so the public,
    /// stable-code set stays exactly the five values the contract pins:
    /// `fileformat.version_too_new`, `fileformat.bad_passphrase`, `fileformat.malformed`,
    /// `fileformat.io`, `fileformat.serde`.
    pub fn code(&self) -> &'static str {
        match self {
            FileFormatError::VersionTooNew { .. } => "fileformat.version_too_new",
            FileFormatError::BadPassphrase => "fileformat.bad_passphrase",
            FileFormatError::Malformed(_) => "fileformat.malformed",
            FileFormatError::Crypto(_) => "fileformat.malformed",
            FileFormatError::Io(_) => "fileformat.io",
            FileFormatError::Serde(_) => "fileformat.serde",
        }
    }
}

/// Map a zip error onto the crate error. An I/O failure stays I/O; everything else (invalid
/// archive, unsupported archive, a missing `manifest.json` on lookup) is `Malformed` — from
/// the caller's perspective the file simply is not a valid `.shalgalt`.
impl From<zip::result::ZipError> for FileFormatError {
    fn from(e: zip::result::ZipError) -> Self {
        match e {
            zip::result::ZipError::Io(io) => FileFormatError::Io(io),
            other => FileFormatError::Malformed(other.to_string()),
        }
    }
}

/// Convenience alias for fallible crate operations.
pub type FileFormatResult<T> = Result<T, FileFormatError>;
