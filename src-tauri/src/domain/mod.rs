//! 인프라(DB / CV / Tauri)에 의존하지 않는 순수 도메인 모델.
//!
//! 프론트엔드와 공유되는 직렬화 단위이며, 다른 모든 모듈은 이 모듈에 의존할 수 있지만
//! 이 모듈은 어떤 모듈에도 의존하지 않는다 (의존 역전 방지).

pub mod result;
pub mod student;
pub mod template;

pub use result::{GradedAnswer, GradedSheet};
pub use student::Student;
pub use template::{BubbleGroup, BubbleKind, Marker, OmrTemplate, TemplatePoint};
