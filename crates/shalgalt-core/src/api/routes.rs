//! axum route definitions for the `/v1/` REST surface (P5-03/04).
//!
//! The router is parameterized over [`AppState`] (a handle to the host's [`DataStore`]) so
//! the identical `Router` runs inside `apps/desktop` (read-only, `127.0.0.1`) and
//! `apps/server` (read-write, `0.0.0.0`). Process concerns — binding, shutdown, auth, CORS
//! — are layered on by the host, never here (Rule 4).
//!
//! Handlers carry `#[utoipa::path]` so the OpenAPI document in [`super::openapi`] stays in
//! lockstep with the wiring below.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use utoipa::ToSchema;

use super::dto::{
    ExamDto, NewExam, NewResult, ResultDto, ResultFilter, TemplateDto, TemplateSummaryDto,
};
use super::state::AppState;
use crate::error::{AppError, AppResult};

/// Health probe payload.
#[derive(Serialize, ToSchema)]
pub(crate) struct Health {
    ok: bool,
    version: &'static str,
}

/// Build the full router. Health + OpenAPI live outside `/v1/` so probes and the spec are
/// reachable without versioning churn.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/openapi.json", get(super::openapi::openapi_json))
        .route("/v1/exams", get(list_exams).post(create_exam))
        .route("/v1/exams/{id}", get(get_exam))
        .route("/v1/templates", get(list_templates))
        .route("/v1/templates/{id}", get(get_template))
        .route("/v1/results", get(list_results).post(create_result))
        .with_state(state)
}

#[utoipa::path(get, path = "/healthz", tag = "meta",
    responses((status = 200, description = "Service is up", body = Health)))]
pub(crate) async fn healthz() -> Json<Health> {
    Json(Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[utoipa::path(get, path = "/v1/exams", tag = "exams",
    responses((status = 200, description = "All exams", body = [ExamDto])))]
pub(crate) async fn list_exams(State(s): State<AppState>) -> AppResult<Json<Vec<ExamDto>>> {
    Ok(Json(s.store.list_exams()?))
}

#[utoipa::path(get, path = "/v1/exams/{id}", tag = "exams",
    params(("id" = i64, Path, description = "Exam id")),
    responses(
        (status = 200, description = "The exam", body = ExamDto),
        (status = 404, description = "No exam with that id")))]
pub(crate) async fn get_exam(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ExamDto>> {
    s.store
        .get_exam(id)?
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("exam {id}")))
}

#[utoipa::path(post, path = "/v1/exams", tag = "exams",
    request_body = NewExam,
    responses(
        (status = 201, description = "Created exam", body = ExamDto),
        (status = 400, description = "Read-only mode or invalid body")))]
pub(crate) async fn create_exam(
    State(s): State<AppState>,
    Json(body): Json<NewExam>,
) -> AppResult<(StatusCode, Json<ExamDto>)> {
    let exam = s.store.create_exam(body)?;
    Ok((StatusCode::CREATED, Json(exam)))
}

#[utoipa::path(get, path = "/v1/templates", tag = "templates",
    responses((status = 200, description = "All templates (metadata only)", body = [TemplateSummaryDto])))]
pub(crate) async fn list_templates(
    State(s): State<AppState>,
) -> AppResult<Json<Vec<TemplateSummaryDto>>> {
    Ok(Json(s.store.list_templates()?))
}

#[utoipa::path(get, path = "/v1/templates/{id}", tag = "templates",
    params(("id" = i64, Path, description = "Template id")),
    responses(
        (status = 200, description = "Template with its serialized OmrTemplate", body = TemplateDto),
        (status = 404, description = "No template with that id")))]
pub(crate) async fn get_template(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<TemplateDto>> {
    s.store
        .get_template(id)?
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("template {id}")))
}

#[utoipa::path(get, path = "/v1/results", tag = "results",
    params(("exam_id" = Option<i64>, Query, description = "Filter by exam id")),
    responses((status = 200, description = "Graded results", body = [ResultDto])))]
pub(crate) async fn list_results(
    State(s): State<AppState>,
    Query(filter): Query<ResultFilter>,
) -> AppResult<Json<Vec<ResultDto>>> {
    Ok(Json(s.store.list_results(filter)?))
}

#[utoipa::path(post, path = "/v1/results", tag = "results",
    request_body = NewResult,
    responses(
        (status = 201, description = "Created result", body = ResultDto),
        (status = 400, description = "Read-only mode or invalid body")))]
pub(crate) async fn create_result(
    State(s): State<AppState>,
    Json(body): Json<NewResult>,
) -> AppResult<(StatusCode, Json<ResultDto>)> {
    let result = s.store.create_result(body)?;
    Ok((StatusCode::CREATED, Json(result)))
}
