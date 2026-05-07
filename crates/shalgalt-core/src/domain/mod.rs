//! Pure domain models with no dependency on infrastructure (DB, CV, Tauri).
//!
//! These types are the serialization unit shared with the frontend. Every other module is
//! allowed to depend on this one, but this module depends on none — preserving an inverted
//! dependency direction.

pub mod progress;
pub mod result;
pub mod student;
pub mod template;

pub use progress::{TaskProgress, TaskStage};
pub use result::{GradedAnswer, GradedSheet};
pub use student::Student;
pub use template::{BubbleGroup, BubbleKind, Marker, OmrTemplate, TemplatePoint};
