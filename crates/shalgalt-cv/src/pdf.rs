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
    ensure_dir(out_dir)?;
    rasterize_pdf_pages(pdf_path, out_dir, 0)
}

/// Create `out_dir` (and parents) if missing. Shared by every rasterization entry point
/// so the batch variants can assume the directory exists before writing page PNGs.
fn ensure_dir(out_dir: &Path) -> AppResult<()> {
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)
            .with_context(|| format!("create raster output dir {}", out_dir.display()))?;
    }
    Ok(())
}

/// Rasterize every page of `pdf_path` into `out_dir`, naming files
/// `page-{start+idx:04}.png` so several sources can share one directory with a single
/// continuous page numbering. Assumes `out_dir` already exists (callers go through
/// [`ensure_dir`]).
fn rasterize_pdf_pages(pdf_path: &Path, out_dir: &Path, start: usize) -> AppResult<Vec<PathBuf>> {
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

        let page_no = start + idx;
        let dest = out_dir.join(format!("page-{page_no:04}.png"));
        image
            .save(&dest)
            .with_context(|| format!("save page raster to {}", dest.display()))?;
        info!("pdfium: rasterized page {page_no} -> {}", dest.display());
        out.push(dest);
    }

    Ok(out)
}

/// Raster-image extensions we accept as a single-page grading source — lowercase,
/// no leading dot. Anything not on this list is treated as a multi-page PDF and
/// routed through pdfium. All of these decode under the `image` crate's default
/// codecs (PNG/JPEG/WebP/BMP/TIFF), so no extra Cargo features are required.
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff"];

/// True when `path` carries a raster-image extension we can grade directly (one
/// page), as opposed to a PDF that must be rasterized through pdfium first.
pub fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Rasterize a single grading *source* — a multi-page PDF or one scanned image — into
/// the `page-{idx:04}.png` convention under `out_dir`. The single-source path used by the
/// answer-key importer (which enforces a one-page contract on top); see
/// [`rasterize_sources`] for the multi-file batch variant.
pub fn rasterize_source(source_path: &Path, out_dir: &Path) -> AppResult<Vec<PathBuf>> {
    rasterize_sources(std::slice::from_ref(&source_path.to_path_buf()), out_dir)
}

/// Rasterize an ordered batch of grading sources — any mix of multi-page PDFs and single
/// images — into one continuous `page-{idx:04}.png` sequence under `out_dir`.
///
/// Page numbering is global across the batch: a 3-page PDF followed by two photos yields
/// `page-0000.png … page-0004.png`. Downstream consumers (the asset-path reconstruction in
/// `scan.rs`, the manual-review preview) therefore treat a multi-file upload exactly like
/// one long PDF — `ParsedSheet.page_index` indexes straight into this list.
///
/// Image sources are re-encoded to a real PNG rather than handed through as-is: every
/// downstream consumer already assumes a PNG in the cache dir, and re-encoding guarantees
/// the `asset://localhost/` response carries an `image/png` content type even when the
/// teacher picked a JPEG/WebP.
pub fn rasterize_sources(sources: &[PathBuf], out_dir: &Path) -> AppResult<Vec<PathBuf>> {
    ensure_dir(out_dir)?;
    let mut pages: Vec<PathBuf> = Vec::new();
    for source in sources {
        let start = pages.len();
        if is_image_path(source) {
            pages.push(transcode_image_to_page_png(source, out_dir, start)?);
        } else {
            pages.extend(rasterize_pdf_pages(source, out_dir, start)?);
        }
    }
    Ok(pages)
}

/// Decode a single image file and re-save it as `page-{start:04}.png` under `out_dir`,
/// mirroring a page produced by [`rasterize_pdf_pages`]. Bad/unreadable images surface as
/// `AppError::BadRequest` so the frontend can show a meaningful message instead of a
/// generic pipeline failure. Assumes `out_dir` already exists.
fn transcode_image_to_page_png(
    image_path: &Path,
    out_dir: &Path,
    start: usize,
) -> AppResult<PathBuf> {
    let image = image::open(image_path).map_err(|e| {
        AppError::BadRequest(format!(
            "could not decode image {}: {e}",
            image_path.display()
        ))
    })?;

    let dest = out_dir.join(format!("page-{start:04}.png"));
    image
        .save(&dest)
        .with_context(|| format!("save image page raster to {}", dest.display()))?;
    info!(
        "image: transcoded {} -> {}",
        image_path.display(),
        dest.display()
    );
    Ok(dest)
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
