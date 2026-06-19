//! Runtime binding to the pdfium dynamic library.
//!
//! pdfium-render loads `libpdfium` (`pdfium.dll` / `libpdfium.so` / `libpdfium.dylib`)
//! at runtime. Where that file lives depends on how the app was packaged:
//!
//! - Windows: shipped flat next to the executable, so the exe-dir probe finds it.
//! - Linux (AppImage) / macOS (`.app`): shipped as a Tauri *resource*, which is NOT
//!   the executable's own directory. The host app resolves its resource directory
//!   via Tauri's `PathResolver` and registers it here with [`set_pdfium_dir`] at
//!   startup, before any rasterization runs.
//!
//! Probe order in [`try_bind`]: registered resource dir → executable dir → system.
//! Keeping this crate Tauri-agnostic (it only ever receives a `PathBuf`) preserves the
//! workspace boundary — no `tauri` dependency leaks into the CV pipeline.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use pdfium_render::prelude::Pdfium;
use tracing::{info, warn};

use shalgalt_core::error::{AppError, AppResult};

/// Directory holding the bundled pdfium library, registered once by the host app.
static PDFIUM_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Register the directory where the bundled pdfium library lives. The host app
/// (which owns Tauri's `PathResolver`) calls this once at startup with its resolved
/// resource directory. A no-op if already set, so it is safe to call defensively.
pub fn set_pdfium_dir(dir: PathBuf) {
    let _ = PDFIUM_DIR.set(dir);
}

/// Try to bind pdfium from `dir/<platform-library-name>`, logging the outcome.
fn bind_in_dir(dir: &Path) -> Option<Pdfium> {
    let candidate = dir.join(Pdfium::pdfium_platform_library_name());
    match Pdfium::bind_to_library(&candidate) {
        Ok(bindings) => {
            info!("pdfium: bound to {}", candidate.display());
            Some(Pdfium::new(bindings))
        }
        Err(e) => {
            warn!(
                "pdfium: bind_to_library({}) failed: {e}",
                candidate.display()
            );
            None
        }
    }
}

/// Bind to a pdfium dynamic library. Tries, in order: the directory registered via
/// [`set_pdfium_dir`] (the host's resource dir), the executable's own directory, then
/// the system library path. Returns [`AppError::PdfiumUnavailable`] if all fail so the
/// UI can show a graceful Mongolian toast instead of crashing.
pub(crate) fn try_bind() -> AppResult<Pdfium> {
    if let Some(dir) = PDFIUM_DIR.get() {
        if let Some(pdfium) = bind_in_dir(dir) {
            return Ok(pdfium);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if let Some(pdfium) = bind_in_dir(exe_dir) {
                return Ok(pdfium);
            }
        }
    }

    match Pdfium::bind_to_system_library() {
        Ok(bindings) => {
            info!("pdfium: bound to system library");
            Ok(Pdfium::new(bindings))
        }
        Err(e) => {
            warn!("pdfium: bind_to_system_library failed: {e}");
            Err(AppError::PdfiumUnavailable)
        }
    }
}
