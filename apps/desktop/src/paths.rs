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

use shalgalt_core::error::{AppError, AppResult};
use tauri::{AppHandle, Manager};

/// SQLite filename. Kept in sync with the `sqlite:` URL passed to `tauri-plugin-sql`.
pub const DB_FILENAME: &str = "shalgalt-omr.sqlite";

/// Bundle of standard directories used by the app.
#[derive(Debug, Clone)]
pub struct AppDirs {
    /// Persistent app data: scan output and graded result images.
    pub data_dir: PathBuf,
    /// App config dir. `tauri-plugin-sql` resolves its `sqlite:` URL relative to this
    /// directory, so the SQLite file lives here (not in `data_dir`).
    pub config_dir: PathBuf,
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
        let config_dir = resolver.app_config_dir().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("failed to resolve app_config_dir: {e}"))
        })?;
        let cache_dir = resolver.app_cache_dir().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("failed to resolve app_cache_dir: {e}"))
        })?;
        let scans_dir = data_dir.join("scans");

        for d in [&data_dir, &config_dir, &cache_dir, &scans_dir] {
            ensure_dir(d)?;
        }

        Ok(Self {
            data_dir,
            config_dir,
            scans_dir,
            cache_dir,
        })
    }

    /// Absolute path to the SQLite file `tauri-plugin-sql` reads and writes. Resolved
    /// against `config_dir` to match the plugin's own `sqlite:` URL resolution (the file
    /// the read-only API reader opens — see `api::server::spawn`).
    pub fn db_path(&self) -> PathBuf {
        self.config_dir.join(DB_FILENAME)
    }
}

fn ensure_dir(p: &Path) -> AppResult<()> {
    if !p.exists() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}
