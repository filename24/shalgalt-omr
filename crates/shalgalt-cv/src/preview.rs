//! PDF first-page rasterization for the P1 template editor backdrop.
//!
//! This is the only piece of P2's CV pipeline pulled forward into P1 — it
//! gives the editor a real visual reference (the scanned blank form) so the
//! teacher can place markers and bubble groups against an actual layout.
//! Heavy work runs on a `tokio::spawn_blocking` thread (Rule 2; pdfium's API
//! is sync), and the resulting PNG is written under the cache dir; only the
//! absolute path crosses the IPC boundary (Rule 1).
//!
//! pdfium-render binds to a dynamic library at runtime. We try the executable's
//! own directory first (matches how P5 will bundle the dylib), then the system
//! library path. If neither resolves, we surface `AppError::PdfiumUnavailable`
//! so the UI can show a graceful Mongolian toast instead of crashing.

use std::path::{Path, PathBuf};

use anyhow::Context;
use pdfium_render::prelude::{PdfPageRenderRotation, PdfRenderConfig, PdfiumError};
use sha2::{Digest, Sha256};
use tracing::info;

use shalgalt_core::error::{AppError, AppResult};

/// Width of the rendered first page in pixels. 2000px gives the editor plenty
/// of detail for marker placement without producing huge PNGs.
const TARGET_WIDTH: i32 = 2000;

/// Stable cache filename derived from the PDF's absolute path. Two imports of
/// the same PDF share a rendered PNG; rebuilding requires deleting the cache
/// dir or invalidating manually.
fn cache_filename(pdf_path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pdf_path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("{}.png", &hex[..16])
}

/// Render the first page of `pdf_path` into a PNG under `<cache_dir>/pdf-preview/`.
///
/// Returns the absolute path to the rendered PNG. The caller (typically the
/// frontend, via `templateAssets.ts`) copies this file into the per-template
/// asset folder under `$APPDATA` so the asset protocol scope sees it.
pub fn rasterize_first_page(pdf_path: &Path, cache_dir: &Path) -> AppResult<PathBuf> {
    let preview_dir = cache_dir.join("pdf-preview");
    if !preview_dir.exists() {
        std::fs::create_dir_all(&preview_dir).context("create pdf-preview cache dir")?;
    }
    let dest = preview_dir.join(cache_filename(pdf_path));

    if dest.exists() {
        info!("pdfium: cache hit for {}", pdf_path.display());
        return Ok(dest);
    }

    let pdfium = crate::binding::try_bind()?;

    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .map_err(map_pdfium_error)?;

    let page = document
        .pages()
        .iter()
        .next()
        .ok_or_else(|| AppError::BadRequest("PDF has no pages".into()))?;

    let config = PdfRenderConfig::new()
        .set_target_width(TARGET_WIDTH)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

    let bitmap = page.render_with_config(&config).map_err(map_pdfium_error)?;
    let image = bitmap.as_image();
    image
        .save(&dest)
        .with_context(|| format!("save preview png to {}", dest.display()))?;

    info!("pdfium: rendered first page to {}", dest.display());
    Ok(dest)
}

fn map_pdfium_error(e: PdfiumError) -> AppError {
    AppError::Internal(anyhow::anyhow!("pdfium: {e}"))
}
