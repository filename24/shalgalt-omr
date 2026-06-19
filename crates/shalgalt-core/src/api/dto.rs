//! Data-transfer objects for the `/v1/` REST surface (P5-03/04).
//!
//! These are the wire shapes the HTTP API speaks — deliberately separate from the
//! `domain::*` models so the REST contract can evolve without dragging the IPC/grading
//! types with it. Every DTO derives `ToSchema` so the same definitions feed the OpenAPI
//! document served at `/openapi.json` (P5-06).
//!
//! `*Dto` types are read responses; `New*` types are write request bodies. The host's
//! `DataStore` implementation (see [`super::store`]) maps between these and the SQLite
//! rows — core never touches a database itself.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// An exam: a named binding of an answer-key set to a template (`exams` table).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExamDto {
    pub id: i64,
    pub name: String,
    pub template_id: i64,
    /// SQLite `CURRENT_TIMESTAMP` text (`YYYY-MM-DD HH:MM:SS`, UTC).
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for `POST /v1/exams`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct NewExam {
    pub name: String,
    pub template_id: i64,
}

/// A template as it appears in `GET /v1/templates` listings — metadata only, without the
/// (potentially large) serialized `OmrTemplate` body.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemplateSummaryDto {
    pub id: i64,
    pub title: String,
    pub created_at: String,
}

/// A template with its full Rule 3 payload, returned by `GET /v1/templates/:id`.
///
/// `json_schema` is the serialized `OmrTemplate` exactly as stored in
/// `templates.json_schema` (TEXT). It is handed back verbatim rather than re-parsed so the
/// API never reshapes the canonical template format — consumers parse it with the same
/// schema the editor wrote.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemplateDto {
    pub id: i64,
    pub title: String,
    pub json_schema: String,
    pub created_at: String,
}

/// A graded result row (`results` table).
///
/// `detail_answers` is opaque JSON (per-question correct/wrong/blank) carried as a string
/// so the API stays agnostic to the grading engine's internal shape — the same posture as
/// `templates.json_schema`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResultDto {
    pub id: i64,
    pub student_id: Option<i64>,
    pub template_id: i64,
    pub exam_id: Option<i64>,
    pub total_score: f64,
    pub detail_answers: String,
    pub image_path: Option<String>,
    pub created_at: String,
}

/// Request body for `POST /v1/results`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct NewResult {
    pub student_id: Option<i64>,
    pub template_id: i64,
    pub exam_id: Option<i64>,
    pub total_score: f64,
    /// Opaque JSON string mirroring [`ResultDto::detail_answers`].
    pub detail_answers: String,
    /// Caller-controlled, untrusted opaque string. The API stores it verbatim and never
    /// touches the filesystem with it; any consumer that turns it into a path (e.g. the
    /// desktop loading it via `asset://`) MUST sanitize it against directory traversal
    /// first — it is not validated here.
    pub image_path: Option<String>,
}

/// Filter for `GET /v1/results`. `exam_id` filters on the `results.exam_id` column
/// directly; absent means "all results".
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ResultFilter {
    pub exam_id: Option<i64>,
}
