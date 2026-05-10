//! Pure scoring engine — `(OmrTemplate, ParsedSheet, AnswerKey) -> GradedSheet`.
//!
//! No CV / I/O code lives here. See [`engine`] for the classification rules.

pub mod engine;

pub use engine::grade;
