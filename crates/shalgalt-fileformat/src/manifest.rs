//! The `manifest.json` header.
//!
//! The manifest is the one entry that is **always plaintext**, even in an encrypted archive,
//! so file managers, the dashboard, and the import dialog can preview a project before
//! prompting for a passphrase. It therefore MUST NOT contain student PII (master plan §6.3).

use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::error::{FileFormatError, FileFormatResult};
use crate::FORMAT_VERSION;

/// Plaintext JSON header stored at the archive root as `manifest.json`.
///
/// Required keys mirror the locked contract (`format_version`, `title`, `created_at`,
/// `sheet_count`, `encrypted`); `exam_id` and `hint` are optional and omitted from the JSON
/// when absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Integer version of the `.shalgalt` format. A loader refuses values greater than
    /// [`FORMAT_VERSION`].
    pub format_version: u32,

    /// Human-readable exam or project title. MUST NOT contain student PII.
    pub title: String,

    /// Creation timestamp, serialized as RFC 3339 / ISO 8601 (e.g. `2026-05-31T09:00:00Z`).
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Number of answer sheets the project expects. `0` means unknown.
    pub sheet_count: u32,

    /// When `true`, every entry except `manifest.json` is age-encrypted (ASCII-armored).
    pub encrypted: bool,

    /// Optional link back to the `exams` row in the local SQLite DB. Absent when the project
    /// is shared across machines — the local row id is not portable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exam_id: Option<i64>,

    /// Optional human-readable hint shown before the passphrase prompt (e.g. `"Spring 2026"`).
    /// MUST NOT contain the passphrase itself or any student data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl Manifest {
    /// Build a minimal manifest for the current format version. `exam_id` / `hint` default to
    /// `None`; set them on the returned value when needed.
    pub fn new(
        title: impl Into<String>,
        created_at: chrono::DateTime<chrono::Utc>,
        sheet_count: u32,
        encrypted: bool,
    ) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            title: title.into(),
            created_at,
            sheet_count,
            encrypted,
            exam_id: None,
            hint: None,
        }
    }

    /// Parse a manifest from `r` and enforce the forward-compatibility version gate.
    ///
    /// This is the single place the version check lives, and it runs before any decryption,
    /// so a future-format archive fails with [`FileFormatError::VersionTooNew`] rather than a
    /// confusing crypto error.
    pub fn from_reader(r: impl Read) -> FileFormatResult<Self> {
        let manifest: Manifest = serde_json::from_reader(r)?;
        if manifest.format_version > FORMAT_VERSION {
            return Err(FileFormatError::VersionTooNew {
                found: manifest.format_version,
                supported: FORMAT_VERSION,
            });
        }
        Ok(manifest)
    }

    /// Serialize the manifest to pretty JSON bytes for storage as the plaintext entry.
    pub fn to_json_bytes(&self) -> FileFormatResult<Vec<u8>> {
        Ok(serde_json::to_vec_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_time() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-05-31T09:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    #[test]
    fn round_trips_through_json() {
        let manifest = Manifest::new("Алгебр I", fixed_time(), 42, true);
        let bytes = manifest.to_json_bytes().unwrap();
        let parsed = Manifest::from_reader(&bytes[..]).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn omits_optional_fields_when_absent() {
        let manifest = Manifest::new("t", fixed_time(), 0, false);
        let json = String::from_utf8(manifest.to_json_bytes().unwrap()).unwrap();
        assert!(!json.contains("exam_id"));
        assert!(!json.contains("hint"));
    }

    #[test]
    fn refuses_future_format_version() {
        let json = r#"{
            "format_version": 999,
            "title": "from the future",
            "created_at": "2026-05-31T09:00:00Z",
            "sheet_count": 0,
            "encrypted": false
        }"#;
        let err = Manifest::from_reader(json.as_bytes()).unwrap_err();
        assert_eq!(err.code(), "fileformat.version_too_new");
        match err {
            FileFormatError::VersionTooNew { found, supported } => {
                assert_eq!(found, 999);
                assert_eq!(supported, FORMAT_VERSION);
            }
            other => panic!("expected VersionTooNew, got {other:?}"),
        }
    }
}
