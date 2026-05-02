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
