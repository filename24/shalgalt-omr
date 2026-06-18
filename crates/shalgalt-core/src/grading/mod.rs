//! Pure scoring engine — `(OmrTemplate, ParsedSheet, AnswerKey) -> GradedSheet`.
//!
//! No CV / I/O code lives here. See [`engine`] for the classification rules.

pub mod engine;
pub mod select;

pub use engine::{awarded_points, grade};
pub use select::select_answer_key;
