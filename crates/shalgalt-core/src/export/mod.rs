//! Serialize results to external formats (currently xlsx).
//!
//! `report` holds the host-facing data model + pure aggregation; `xlsx` is the
//! `rust_xlsxwriter` presentation layer. Human-language text is injected via
//! [`report::ReportLabels`] so this crate stays English-only (see AGENTS.md).

pub mod report;
pub mod xlsx;

pub use report::{QuestionColumn, ReportLabels, StudentRow, XlsxReport};
pub use xlsx::build_report;
