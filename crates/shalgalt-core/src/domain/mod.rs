//! Pure domain models with no dependency on infrastructure (DB, CV, Tauri).
//!
//! These types are the serialization unit shared with the frontend. Every other module is
//! allowed to depend on this one, but this module depends on none — preserving an inverted
//! dependency direction.

pub mod answer_key;
pub mod paper;
pub mod parsed;
pub mod progress;
pub mod result;
pub mod student;
pub mod template;

pub use answer_key::{AnswerKey, AnswerKeyEntry};
pub use paper::{Orientation, PaperSpec};
pub use parsed::{BubbleReading, ParsedSheet};
pub use progress::{TaskProgress, TaskStage};
pub use result::{GradedAnswer, GradedSheet};
pub use student::Student;
pub use template::{BubbleGroup, BubbleKind, Marker, MarkerKind, OmrTemplate, TemplatePoint};
