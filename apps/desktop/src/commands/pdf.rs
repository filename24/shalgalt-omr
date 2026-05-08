//! PDF generation IPC.
//!
//! Two commands wrap [`shalgalt_pdf::render_template`]:
//!
//! - `pdf_generate_omr` writes a freshly rendered PDF to a user-picked path
//!   (P2-07). Rule 1: only the absolute output path crosses the IPC boundary.
//! - `pdf_render_template_preview` renders to a cache-dir scratch PDF, then
//!   rasterizes the first page to a PNG via `shalgalt_cv::preview` and returns
//!   the asset-protocol path the editor pane consumes (P2-08).
//!
//! Rule 2: `printpdf` and `pdfium-render` both have synchronous APIs, so the
//! heavy work runs inside `tokio::task::spawn_blocking`.

use std::path::PathBuf;

use sha2::{Digest, Sha256};
use shalgalt_core::domain::template::OmrTemplate;
use shalgalt_core::error::{AppError, AppResult};
use shalgalt_cv::preview;
use shalgalt_pdf::{render_template, PdfOptions};
use tauri::State;

use crate::state::AppState;

/// Generate an OMR PDF from a serialized template and write it to `output_path`.
///
/// `template_json` is the serialized [`OmrTemplate`] (Rule 3 unit). `variant`
/// flows through to [`PdfOptions::variant`] for multi-variant exams. The
/// command returns once the bytes are flushed to disk; the toolbar surfaces a
/// success toast or maps the typed error code to a Mongolian message.
#[tauri::command]
pub async fn pdf_generate_omr(
    _state: State<'_, AppState>,
    template_json: String,
    variant: Option<String>,
    output_path: String,
) -> AppResult<()> {
    if output_path.trim().is_empty() {
        return Err(AppError::BadRequest("output_path is empty".into()));
    }

    let template: OmrTemplate = serde_json::from_str(&template_json)
        .map_err(|e| AppError::BadRequest(format!("invalid template_json: {e}")))?;

    let bytes = render_pdf_blocking(template, variant).await?;

    tokio::fs::write(&output_path, &bytes)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("write {output_path}: {e}")))?;

    tracing::info!(output_path = %output_path, bytes = bytes.len(), "wrote OMR PDF");
    Ok(())
}

/// Render the current template into a cached PDF, rasterize the first page,
/// and return the absolute PNG path. The frontend feeds the path through
/// `convertFileSrc` and shows it in a `<img>`. Rule 1: the bytes never cross
/// the IPC boundary.
#[tauri::command]
pub async fn pdf_render_template_preview(
    state: State<'_, AppState>,
    template_json: String,
    variant: Option<String>,
) -> AppResult<String> {
    let template: OmrTemplate = serde_json::from_str(&template_json)
        .map_err(|e| AppError::BadRequest(format!("invalid template_json: {e}")))?;

    let bytes = render_pdf_blocking(template, variant.clone()).await?;

    let cache_dir = state.dirs().cache_dir.clone();
    let preview_dir = cache_dir.join("preview");
    tokio::fs::create_dir_all(&preview_dir)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("mkdir preview: {e}")))?;

    let hash = preview_cache_key(&bytes, variant.as_deref());
    let pdf_path = preview_dir.join(format!("{hash}.pdf"));
    tokio::fs::write(&pdf_path, &bytes)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("write preview pdf: {e}")))?;

    let png_path =
        tokio::task::spawn_blocking(move || preview::rasterize_first_page(&pdf_path, &cache_dir))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("join error: {e}")))??;

    Ok(png_path.to_string_lossy().into_owned())
}

/// Run [`render_template`] on a blocking thread so the async runtime stays
/// free during PDF generation (Rule 2).
async fn render_pdf_blocking(template: OmrTemplate, variant: Option<String>) -> AppResult<Vec<u8>> {
    let opts = PdfOptions {
        variant,
        ..PdfOptions::default()
    };
    tokio::task::spawn_blocking(move || render_template(&template, &opts))
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("join error: {e}")))?
        .map_err(|e| AppError::Internal(anyhow::anyhow!("pdf render failed: {e}")))
}

/// SHA-256 over the rendered PDF bytes plus the variant tag. Two renders that
/// produce identical bytes share a cache file; tweaking the variant or any
/// template field invalidates the cache automatically because `created_at`
/// defaults to a deterministic epoch.
fn preview_cache_key(bytes: &[u8], variant: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    if let Some(v) = variant {
        hasher.update(b"|variant=");
        hasher.update(v.as_bytes());
    }
    let digest = hasher.finalize();
    digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()[..32]
        .to_string()
}

/// Best-effort cleanup of preview artifacts older than 24 hours. Called once
/// during boot so the cache directory does not grow unbounded across sessions.
pub fn prune_old_previews(cache_dir: &std::path::Path) {
    let preview_dir = cache_dir.join("preview");
    let Ok(entries) = std::fs::read_dir(&preview_dir) else {
        return;
    };
    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 24);
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        if modified < cutoff {
            let path: PathBuf = entry.path();
            if let Err(e) = std::fs::remove_file(&path) {
                tracing::warn!(path = %path.display(), "preview prune failed: {e}");
            }
        }
    }
}
