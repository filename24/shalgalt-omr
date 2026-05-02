//! Helper for resolving OS-specific data / cache / scan output directories.
//!
//! Rule 1: result images live in a temp directory and are exposed to the UI through the
//! `asset://` protocol, so every path decision flows through this single source.

use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::error::{AppError, AppResult};

/// SQLite filename. Kept in sync with the `sqlite:` URL passed to `tauri-plugin-sql`.
pub const DB_FILENAME: &str = "shalgalt-omr.sqlite";

/// Bundle of standard directories used by the app.
#[derive(Debug, Clone)]
pub struct AppDirs {
    /// SQLite file and persistent settings.
    pub data_dir: PathBuf,
    /// PDF rasterization output and graded result images (exposed via `asset://`).
    pub scans_dir: PathBuf,
    /// One-off temp files.
    pub cache_dir: PathBuf,
}

impl AppDirs {
    /// `directories` picks the appropriate location per OS.
    pub fn resolve() -> AppResult<Self> {
        let dirs = ProjectDirs::from("dev", "filename", "shalgalt-omr").ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "failed to resolve project directories for current OS"
            ))
        })?;

        let data_dir = dirs.data_dir().to_path_buf();
        let cache_dir = dirs.cache_dir().to_path_buf();
        let scans_dir = data_dir.join("scans");

        for d in [&data_dir, &cache_dir, &scans_dir] {
            ensure_dir(d)?;
        }

        Ok(Self {
            data_dir,
            scans_dir,
            cache_dir,
        })
    }

    /// Absolute path to the SQLite file.
    /// (Reference only — actual DB I/O is performed by `tauri-plugin-sql` via its
    /// AppConfig-relative URL.)
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join(DB_FILENAME)
    }
}

fn ensure_dir(p: &Path) -> AppResult<()> {
    if !p.exists() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}
