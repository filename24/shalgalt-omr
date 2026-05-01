//! 컴퓨터 비전 채점 파이프라인.
//!
//! Blueprint Rule 1: 입력은 항상 **로컬 절대 경로 문자열**로 받고, 결과 이미지는
//! 임시 폴더에 저장 후 경로만 반환한다 (Base64 금지).
//! Blueprint Rule 2: 모든 무거운 연산은 `tokio::spawn` 백그라운드 task에서 수행하며,
//! `app_handle.emit("task-progress", ...)` 로 진행률을 발행한다.
//!
//! P0에서는 인터페이스만 정의한다. 실제 구현(opencv-rust, pdfium-render 호출)은 P2에서 채운다.

pub mod bubbles;
pub mod pdf;
pub mod perspective;
pub mod pipeline;

pub use pipeline::{TaskProgress, TaskStage};
