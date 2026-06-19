//! OpenAPI 3.1 document for the `/v1/` surface (P5-06).
//!
//! The spec is derived from the same `#[utoipa::path]` annotations on the route handlers,
//! so it cannot drift from the wiring. It is served verbatim at `GET /openapi.json`; no
//! Swagger UI is bundled (this is an offline-first app — consumers point their own tooling
//! at the JSON).

use axum::Json;
use utoipa::OpenApi;

use super::dto::{ExamDto, NewExam, NewResult, ResultDto, TemplateDto, TemplateSummaryDto};
use super::routes::Health;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "shalgalt-omr HTTP API",
        version = "1.0",
        description = "Read/write access to exams, templates, and graded results. The \
                       desktop app exposes this read-only on 127.0.0.1; the standalone \
                       server exposes it read-write with bearer auth."
    ),
    paths(
        super::routes::healthz,
        super::routes::list_exams,
        super::routes::get_exam,
        super::routes::create_exam,
        super::routes::list_templates,
        super::routes::get_template,
        super::routes::list_results,
        super::routes::create_result,
    ),
    components(schemas(
        Health,
        ExamDto,
        NewExam,
        TemplateSummaryDto,
        TemplateDto,
        ResultDto,
        NewResult,
    )),
    tags(
        (name = "meta", description = "Health and service metadata"),
        (name = "exams", description = "Exam definitions"),
        (name = "templates", description = "OMR templates"),
        (name = "results", description = "Graded results"),
    )
)]
pub struct ApiDoc;

/// Handler for `GET /openapi.json`.
pub(crate) async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
