//! `.shalgalt` project-file IPC.
//!
//! Three commands wrap [`shalgalt_fileformat`]:
//!
//! - `project_save` — write the current project to its own on-disk path (overwrite-in-place).
//! - `project_export` — write the project to a user-picked Save-As path (a share copy).
//!   Functionally identical to `project_save` at the Rust layer; the distinct command name
//!   keeps the master-plan contract intact and lets the frontend label the two intents.
//! - `project_open` — extract an archive into a per-input scratch workspace under the cache
//!   dir and return the parsed manifest plus the extracted entry paths.
//!
//! Rule 1: large payloads (scanned PDFs, images) cross the IPC boundary as path strings only.
//! Inline entries are reserved for small JSON/CSV the frontend already holds in memory; binary
//! entries are streamed from their `source_path` on disk. Extraction streams each entry to a
//! file under the workspace dir — the whole archive is never buffered.
//!
//! Rule 2: `shalgalt_fileformat::{write, open}` are synchronous (zip + age), so the heavy work
//! runs inside `tokio::task::spawn_blocking`.

use std::fs::File;
use std::path::Path;

use sha2::{Digest, Sha256};
use shalgalt_core::error::{AppError, AppResult};
use shalgalt_fileformat::{FileFormatError, Manifest, ReadHandle};
use tauri::State;

use crate::state::AppState;

/// A project entry supplied by the frontend.
///
/// `camelCase` so JS sends `{ kind: "inline", name, content }` or
/// `{ kind: "file", name, sourcePath }`. Inline entries carry small text payloads the UI
/// already holds (template / answer-key / metadata JSON, roster CSV); file entries point at a
/// large binary on disk (the original exam PDF, scan images) that must be streamed (Rule 1).
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProjectEntry {
    /// Small in-memory payload — written via [`ReadHandle::from_bytes`].
    Inline { name: String, content: String },
    /// Large binary on disk — streamed via [`ReadHandle::from_path`] (Rule 1).
    File { name: String, source_path: String },
}

/// One entry that was streamed out of an opened archive onto disk.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedEntry {
    name: String,
    path: String,
}

/// The result of opening a `.shalgalt` archive: the parsed (always-plaintext) manifest, the
/// scratch workspace the entries were extracted into, and the per-entry paths.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedProject {
    /// Serializes with snake_case fields (`format_version`, `created_at`, …) — its own derive.
    manifest: Manifest,
    workspace_dir: String,
    entries: Vec<ExtractedEntry>,
}

/// Map a fileformat error into [`AppError::FileFormat`], preserving the stable `fileformat.*`
/// code so the frontend string-table keys on it. The message is for logs/diagnostics only.
fn map_ff(e: FileFormatError) -> AppError {
    AppError::FileFormat {
        code: e.code(),
        message: e.to_string(),
    }
}

/// Write a project archive to `dest`.
///
/// Sets `manifest.encrypted = passphrase.is_some()` so the writer's encrypted-flag invariant
/// can never trip regardless of what the caller stored on the manifest. Each [`ProjectEntry`]
/// becomes a streamed `(name, ReadHandle)`: inline payloads wrap their bytes, file payloads
/// stream from disk (Rule 1). A bad `source_path` surfaces as a fileformat IO error.
fn write_archive(
    dest: &Path,
    mut manifest: Manifest,
    entries: Vec<ProjectEntry>,
    passphrase: Option<&str>,
) -> Result<(), FileFormatError> {
    manifest.encrypted = passphrase.is_some();

    // Materialize the entry iterator. `from_path` can fail (missing source), so each item is a
    // `Result`; the writer short-circuits on the first error.
    let items = entries.into_iter().map(|entry| match entry {
        ProjectEntry::Inline { name, content } => {
            Ok((name, ReadHandle::from_bytes(content.into_bytes())))
        }
        ProjectEntry::File { name, source_path } => {
            ReadHandle::from_path(&source_path).map(|handle| (name, handle))
        }
    });

    shalgalt_fileformat::write(dest, &manifest, items, passphrase)
}

/// Extract every entry of `src` into `workspace_dir`, streaming one entry at a time.
///
/// Never buffers a whole entry: each handle is piped to a fresh file via [`std::io::copy`].
/// Entry names may contain `/`, so parent directories are created on demand. Returns the
/// parsed manifest plus the on-disk path of every extracted entry.
fn extract_archive(
    src: &Path,
    workspace_dir: &Path,
    passphrase: Option<&str>,
) -> Result<(Manifest, Vec<ExtractedEntry>), FileFormatError> {
    std::fs::create_dir_all(workspace_dir)?;

    let (manifest, mut iter) = shalgalt_fileformat::open(src, passphrase)?;

    let mut extracted = Vec::new();
    while let Some(res) = iter.next_entry() {
        let (name, mut handle) = res?;
        let dest = workspace_dir.join(&name);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = File::create(&dest)?;
        std::io::copy(&mut handle, &mut out)?;
        extracted.push(ExtractedEntry {
            name,
            path: dest.to_string_lossy().into_owned(),
        });
    }

    Ok((manifest, extracted))
}

/// Parse the frontend-supplied manifest JSON into a [`Manifest`], mapping failures to a
/// `bad_request` so the UI can surface a clear validation message.
fn parse_manifest(manifest_json: &str) -> AppResult<Manifest> {
    serde_json::from_str(manifest_json)
        .map_err(|e| AppError::BadRequest(format!("invalid manifest_json: {e}")))
}

/// Save the project to `output_path`, overwriting the project's own file in place.
///
/// Use `project_export` for a Save-As share copy; the two are functionally identical at the
/// Rust layer (same write path) but carry distinct command names per the master-plan contract.
#[tauri::command]
pub async fn project_save(
    _state: State<'_, AppState>,
    manifest_json: String,
    entries: Vec<ProjectEntry>,
    output_path: String,
    passphrase: Option<String>,
) -> AppResult<()> {
    save_archive(manifest_json, entries, output_path, passphrase).await
}

/// Export the project to a user-picked Save-As path (a share copy).
///
/// Delegates to the same write path as `project_save`. Distinct command name so the frontend
/// can label "save" (overwrite own path) vs "export" (Save-As) and the master-plan command
/// surface stays intact; at the Rust layer the behavior is identical.
#[tauri::command]
pub async fn project_export(
    _state: State<'_, AppState>,
    manifest_json: String,
    entries: Vec<ProjectEntry>,
    output_path: String,
    passphrase: Option<String>,
) -> AppResult<()> {
    save_archive(manifest_json, entries, output_path, passphrase).await
}

/// Shared body for `project_save` / `project_export`: validate input, parse the manifest, then
/// stream the archive to disk on a blocking thread (Rule 2).
async fn save_archive(
    manifest_json: String,
    entries: Vec<ProjectEntry>,
    output_path: String,
    passphrase: Option<String>,
) -> AppResult<()> {
    if output_path.trim().is_empty() {
        return Err(AppError::BadRequest("output_path is empty".into()));
    }

    let manifest = parse_manifest(&manifest_json)?;
    let dest = std::path::PathBuf::from(&output_path);

    tokio::task::spawn_blocking(move || {
        write_archive(&dest, manifest, entries, passphrase.as_deref())
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("join error: {e}")))?
    .map_err(map_ff)?;

    tracing::info!(output_path = %output_path, "wrote .shalgalt project");
    Ok(())
}

/// Open a `.shalgalt` archive into a per-input scratch workspace and return its manifest +
/// extracted entry paths.
///
/// The workspace id is the first 32 hex chars of the SHA-256 of `input_path`, so reopening the
/// same file reuses the same workspace dir (matching the preview-cache key scheme in
/// `commands::pdf`). Rule 1: only paths cross the IPC boundary; bytes are streamed to disk.
/// Rule 2: extraction runs on a blocking thread.
#[tauri::command]
pub async fn project_open(
    state: State<'_, AppState>,
    input_path: String,
    passphrase: Option<String>,
) -> AppResult<OpenedProject> {
    if input_path.trim().is_empty() {
        return Err(AppError::BadRequest("input_path is empty".into()));
    }

    let id = workspace_id(&input_path);
    let workspace_dir = state.dirs().cache_dir.join("workspace").join(id);

    let src = std::path::PathBuf::from(&input_path);
    let dir = workspace_dir.clone();
    let (manifest, entries) =
        tokio::task::spawn_blocking(move || extract_archive(&src, &dir, passphrase.as_deref()))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("join error: {e}")))?
            .map_err(map_ff)?;

    tracing::info!(
        input_path = %input_path,
        entries = entries.len(),
        "opened .shalgalt project"
    );

    Ok(OpenedProject {
        manifest,
        workspace_dir: workspace_dir.to_string_lossy().into_owned(),
        entries,
    })
}

/// First 32 hex chars of the SHA-256 of the input path — a stable, filesystem-safe workspace
/// id. Same scheme as `commands::pdf::preview_cache_key` so the two cache layouts are uniform.
fn workspace_id(input_path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input_path.as_bytes());
    let digest = hasher.finalize();
    digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()[..32]
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fixed_time() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-05-31T09:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    /// Write `bytes` to a temp file and return its path; used to exercise the streamed
    /// `ProjectEntry::File` path through `write_archive`.
    fn temp_source(dir: &std::path::Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(bytes).unwrap();
        path
    }

    fn read_extracted(entries: &[ExtractedEntry], name: &str) -> Vec<u8> {
        let entry = entries.iter().find(|e| e.name == name).unwrap();
        std::fs::read(&entry.path).unwrap()
    }

    #[test]
    fn plaintext_round_trip_preserves_bytes_and_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("project.shalgalt");

        let pdf_bytes = b"%PDF-1.7 fake exam bytes".to_vec();
        let src = temp_source(tmp.path(), "exam.pdf", &pdf_bytes);

        let entries = vec![
            ProjectEntry::Inline {
                name: "template.json".into(),
                content: r#"{"id":"t1"}"#.into(),
            },
            ProjectEntry::File {
                name: "exam.pdf".into(),
                source_path: src.to_string_lossy().into_owned(),
            },
        ];

        let manifest = Manifest::new("Алгебр I", fixed_time(), 0, false);
        write_archive(&archive, manifest, entries, None).unwrap();

        let workspace = tmp.path().join("out");
        let (manifest, extracted) = extract_archive(&archive, &workspace, None).unwrap();

        assert_eq!(manifest.title, "Алгебр I");
        assert!(!manifest.encrypted);
        assert_eq!(
            read_extracted(&extracted, "template.json"),
            br#"{"id":"t1"}"#.to_vec()
        );
        assert_eq!(read_extracted(&extracted, "exam.pdf"), pdf_bytes);
    }

    #[test]
    fn encrypted_round_trip_with_correct_passphrase() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("project.shalgalt");

        let entries = vec![ProjectEntry::Inline {
            name: "template.json".into(),
            content: "secret-payload".into(),
        }];

        let manifest = Manifest::new("Encrypted", fixed_time(), 0, false);
        write_archive(&archive, manifest, entries, Some("pw")).unwrap();

        let workspace = tmp.path().join("out");
        let (manifest, extracted) = extract_archive(&archive, &workspace, Some("pw")).unwrap();

        assert!(manifest.encrypted);
        assert_eq!(
            read_extracted(&extracted, "template.json"),
            b"secret-payload".to_vec()
        );
    }

    #[test]
    fn wrong_passphrase_reports_bad_passphrase_code() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("project.shalgalt");

        let entries = vec![ProjectEntry::Inline {
            name: "template.json".into(),
            content: "secret-payload".into(),
        }];

        let manifest = Manifest::new("Encrypted", fixed_time(), 0, false);
        write_archive(&archive, manifest, entries, Some("pw")).unwrap();

        let workspace = tmp.path().join("out");
        let err = extract_archive(&archive, &workspace, Some("wrong")).unwrap_err();
        assert_eq!(err.code(), "fileformat.bad_passphrase");
    }

    #[test]
    fn write_archive_overrides_encrypted_flag_from_passphrase() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("project.shalgalt");

        let entries = vec![ProjectEntry::Inline {
            name: "template.json".into(),
            content: "payload".into(),
        }];

        // Deliberately set the WRONG encrypted flag: manifest says encrypted while we pass no
        // passphrase. `write_archive` overrides it from `passphrase.is_some()`, so the writer's
        // own `manifest.encrypted == passphrase.is_some()` invariant never trips.
        let manifest = Manifest::new("Mismatch", fixed_time(), 0, true);
        write_archive(&archive, manifest, entries, None).unwrap();

        let workspace = tmp.path().join("out");
        let (manifest, _entries) = extract_archive(&archive, &workspace, None).unwrap();
        assert!(!manifest.encrypted);
    }
}
