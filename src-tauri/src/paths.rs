//! Helper for resolving OS-specific data / cache / scan output directories.
//!
//! Rule 1: result images live in a temp directory and are exposed to the UI through the
//! `asset://` protocol, so every path decision flows through this single source.
//!
//! Paths are resolved through Tauri's `PathResolver` so that the Rust backend and the
//! frontend `@tauri-apps/api/path` helpers always agree on locations. This keeps
//! `tauri-plugin-fs` scopes (`$APPDATA/**`, `$APPCACHE/**`) valid for paths produced on
//! either side of the IPC boundary.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

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
    /// Resolve all standard directories via Tauri's path resolver, ensuring they exist.
    pub fn resolve(app: &AppHandle) -> AppResult<Self> {
        let resolver = app.path();
        let data_dir = resolver.app_data_dir().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("failed to resolve app_data_dir: {e}"))
        })?;
        let cache_dir = resolver.app_cache_dir().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("failed to resolve app_cache_dir: {e}"))
        })?;
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
