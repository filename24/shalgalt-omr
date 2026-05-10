//! PDF → high-resolution per-page image conversion via `pdfium-render`.
//!
//! Each page is rasterized to a PNG file under `<cache_dir>/page-rasters/<task_id>/`
//! at a fixed DPI. Only filesystem paths cross the IPC boundary (Rule 1); we never
//! return raw pixel buffers to the desktop or web layer.
//!
//! pdfium-render's API is synchronous, so this module is meant to be called from
//! `tokio::task::spawn_blocking` by the orchestrating pipeline.

use std::path::{Path, PathBuf};

use anyhow::Context;
use pdfium_render::prelude::{PdfRenderConfig, Pdfium, PdfiumError};
use tracing::{info, warn};

use shalgalt_core::error::{AppError, AppResult};

/// Target DPI for grading rasterization. 200 DPI gives ~2480×1750 px on A4 portrait,
/// which is enough for stable bubble fill measurement without exploding cache size.
pub const TARGET_DPI: f32 = 200.0;

/// Render every page of `pdf_path` to PNG under `<out_dir>/page-<n>.png` and return the
/// list of paths in source order.
pub fn rasterize_all_pages(pdf_path: &Path, out_dir: &Path) -> AppResult<Vec<PathBuf>> {
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)
            .with_context(|| format!("create raster output dir {}", out_dir.display()))?;
    }

    let pdfium = try_bind()?;

    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .map_err(map_pdfium_error)?;

    let pages = document.pages();
    let total = pages.len();
    let mut out = Vec::with_capacity(total as usize);

    // A4 portrait at 200 DPI is the default body size; we let pdfium compute the actual
    // pixel dimensions per page rather than hard-coding A4.
    for (idx, page) in pages.iter().enumerate() {
        // pdfium-render returns `PdfPoints`; one PDF point = 1/72 inch.
        let page_width_pt: f32 = page.width().value;
        let page_width_mm = (page_width_pt / 72.0 * 25.4).max(1.0);
        let target_w_px = (page_width_mm / 25.4 * TARGET_DPI) as i32;

        let cfg = PdfRenderConfig::new().set_target_width(target_w_px);
        let bitmap = page.render_with_config(&cfg).map_err(map_pdfium_error)?;
        let image = bitmap.as_image();

        let dest = out_dir.join(format!("page-{idx:04}.png"));
        image
            .save(&dest)
            .with_context(|| format!("save page raster to {}", dest.display()))?;
        info!("pdfium: rasterized page {idx} -> {}", dest.display());
        out.push(dest);
    }

    Ok(out)
}

/// Bind to a pdfium dynamic library, trying the executable's directory first and then
/// the system path. Returns `AppError::PdfiumUnavailable` if neither strategy succeeds.
fn try_bind() -> AppResult<Pdfium> {
    let library_name = Pdfium::pdfium_platform_library_name();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join(&library_name);
            match Pdfium::bind_to_library(&candidate) {
                Ok(bindings) => {
                    info!("pdfium: bound to {}", candidate.display());
                    return Ok(Pdfium::new(bindings));
                }
                Err(e) => {
                    warn!(
                        "pdfium: bind_to_library({}) failed: {e}",
                        candidate.display()
                    );
                }
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

fn map_pdfium_error(e: PdfiumError) -> AppError {
    AppError::Internal(anyhow::anyhow!("pdfium: {e}"))
}
