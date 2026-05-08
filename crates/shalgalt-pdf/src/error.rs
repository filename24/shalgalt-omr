//! Error type exposed at the crate boundary.
//!
//! Internal helpers are free to use `anyhow::Error`, but every public API converts to
//! [`PdfError`] before returning (P2-05 acceptance criterion: no `anyhow` leakage).

use thiserror::Error;

/// Every failure mode the PDF render pipeline can emit.
#[derive(Debug, Error)]
pub enum PdfError {
    /// The embedded TTF font failed to parse — typically a corrupted file or a TTC
    /// index mismatch.
    #[error("failed to load embedded font {font}: {message}")]
    FontLoad {
        /// Static label identifying which font failed.
        font: &'static str,
        /// Parser error message — `printpdf` returns `Option<_>`, so we synthesize a
        /// string here rather than wrapping a typed source.
        message: String,
    },

    /// `printpdf` rejected an `ops` sequence while painting the page.
    #[error("PDF render failed: {reason}")]
    Render {
        /// Human-readable reason.
        reason: String,
    },

    /// The serialized output failed a sanity check (e.g. missing `%PDF-` header).
    #[error("PDF encode failed: {reason}")]
    Encode {
        /// Human-readable reason.
        reason: String,
    },
}
