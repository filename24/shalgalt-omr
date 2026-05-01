//! 앱 전역 에러 타입.
//!
//! Rule 1·2를 안전하게 지키려면 IPC 경계에서 에러도 평문으로 직렬화 가능해야 한다.
//! `AppError`는 백엔드 내부에서 `thiserror`로 풍부한 컨텍스트를 보존하고,
//! `serde::Serialize`를 통해 프론트엔드로 전달될 때는 안전한 메시지/코드만 노출한다.

use serde::Serialize;
use thiserror::Error;

/// 앱 전역 에러.
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

/// IPC 프론트엔드로 보낼 직렬화 안전 표현.
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
