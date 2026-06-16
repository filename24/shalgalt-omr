//! Application-wide error type.
//!
//! To respect Rule 1 and Rule 2 cleanly, errors crossing the IPC boundary must serialize to
//! a frontend-safe payload. `AppError` keeps rich context internally via `thiserror`, while
//! `serde::Serialize` exposes only a stable `{ code, message }` envelope to the UI.

use serde::Serialize;
use thiserror::Error;

/// Application-wide error.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid input: {0}")]
    BadRequest(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("pdfium dynamic library unavailable")]
    PdfiumUnavailable,

    /// Answer-key scan was given a multi-page PDF. The canonical answer sheet must be a
    /// single page (P4-06). Carries no detail — the message is fixed and user-facing copy
    /// is keyed off the stable `code()`.
    #[error("answer key sheet must be a single page")]
    AnswerKeyMultiPage,

    /// Answer-key scan could not locate the four ArUco corner markers, so the page cannot
    /// be aligned to the template (P4-06). The string is diagnostic only.
    #[error("answer key markers not detected: {0}")]
    AnswerKeyMarkerMissing(String),

    /// The scanned sheet does not match the template it was read against — e.g. the
    /// template defines no question groups to read (P4-06). Diagnostic string only.
    #[error("answer key does not match template: {0}")]
    AnswerKeyTemplateMismatch(String),

    /// A `.shalgalt` file-format error surfaced across the IPC boundary. The desktop command
    /// layer maps `shalgalt_fileformat::FileFormatError` into this variant, preserving the
    /// crate's stable `fileformat.*` `code` so the frontend string-table keys on it directly.
    /// Carried as a `&'static str` + owned message so `shalgalt-core` need not depend on the
    /// fileformat crate.
    #[error("{message}")]
    FileFormat { code: &'static str, message: String },
}

/// Serialization-safe representation sent to the IPC frontend.
#[derive(Debug, Serialize)]
pub struct AppErrorPayload {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            AppError::Io(_) => "io_error",
            AppError::Serde(_) => "serde_error",
            AppError::BadRequest(_) => "bad_request",
            AppError::NotFound(_) => "not_found",
            AppError::Internal(_) => "internal",
            AppError::PdfiumUnavailable => "pdfium_unavailable",
            AppError::AnswerKeyMultiPage => "answer_key_multi_page",
            AppError::AnswerKeyMarkerMissing(_) => "answer_key_marker_missing",
            AppError::AnswerKeyTemplateMismatch(_) => "answer_key_template_mismatch",
            AppError::FileFormat { code, .. } => code,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        AppErrorPayload {
            code: self.code(),
            message: self.to_string(),
        }
        .serialize(s)
    }
}

pub type AppResult<T> = Result<T, AppError>;
