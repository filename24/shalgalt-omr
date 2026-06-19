//! The `DataStore` seam between the stateless `/v1/` router and whatever persistence the
//! host provides.
//!
//! Core stays database-free (see the crate `CLAUDE.md`): it defines this trait and the
//! DTOs that cross it, and each host injects a concrete implementation. `apps/server`
//! injects a read-write `rusqlite` store; `apps/desktop` injects a *read-only* one over
//! the same SQLite file `tauri-plugin-sql` owns, so the desktop's HTTP surface can never
//! race the plugin-sql writer (Rule: plugin-sql owns writes). Read-only stores reject the
//! `create_*` methods with [`AppError::BadRequest`].
//!
//! [`MemoryStore`] is an in-process implementation used by the router tests and as a
//! placeholder host store; it keeps the trait honest without a real database.

use std::sync::Mutex;

use crate::error::{AppError, AppResult};

use super::dto::{
    ExamDto, NewExam, NewResult, ResultDto, ResultFilter, TemplateDto, TemplateSummaryDto,
};

/// Persistence operations backing the `/v1/` REST surface. Object-safe so it can live
/// behind `Arc<dyn DataStore>` in [`super::state::AppState`].
pub trait DataStore: Send + Sync {
    fn list_exams(&self) -> AppResult<Vec<ExamDto>>;
    fn get_exam(&self, id: i64) -> AppResult<Option<ExamDto>>;
    fn list_templates(&self) -> AppResult<Vec<TemplateSummaryDto>>;
    fn get_template(&self, id: i64) -> AppResult<Option<TemplateDto>>;
    fn list_results(&self, filter: ResultFilter) -> AppResult<Vec<ResultDto>>;

    fn create_exam(&self, input: NewExam) -> AppResult<ExamDto>;
    fn create_result(&self, input: NewResult) -> AppResult<ResultDto>;
}

/// Stable rejection used by read-only host stores for the `create_*` methods.
pub fn read_only() -> AppError {
    AppError::BadRequest("the HTTP API is read-only in this mode".to_string())
}

/// In-memory `DataStore` for tests and as a no-database placeholder.
///
/// Exams, templates, and results live in `Mutex<Vec<_>>` with monotonically increasing
/// ids. Writes are enabled by default; [`MemoryStore::read_only`] flips it into the
/// read-only posture a desktop host would inject.
#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<MemoryState>,
    read_only: bool,
}

#[derive(Default)]
struct MemoryState {
    exams: Vec<ExamDto>,
    templates: Vec<TemplateDto>,
    results: Vec<ResultDto>,
    next_id: i64,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// A store that rejects writes, mirroring a desktop (plugin-sql-owned) host.
    pub fn read_only() -> Self {
        Self {
            read_only: true,
            ..Self::default()
        }
    }

    /// Seed a template (test helper). Returns the assigned id.
    pub fn seed_template(&self, title: &str, json_schema: &str) -> i64 {
        let mut state = self.inner.lock().expect("memory store poisoned");
        let id = state.bump_id();
        state.templates.push(TemplateDto {
            id,
            title: title.to_string(),
            json_schema: json_schema.to_string(),
            created_at: FIXED_TS.to_string(),
        });
        id
    }
}

/// Fixed timestamp for deterministic test fixtures (core cannot read the clock; that is a
/// host concern, and tests must not depend on wall-clock time).
const FIXED_TS: &str = "2025-01-01 00:00:00";

impl MemoryState {
    fn bump_id(&mut self) -> i64 {
        self.next_id += 1;
        self.next_id
    }
}

impl DataStore for MemoryStore {
    fn list_exams(&self) -> AppResult<Vec<ExamDto>> {
        Ok(self.inner.lock().expect("poisoned").exams.clone())
    }

    fn get_exam(&self, id: i64) -> AppResult<Option<ExamDto>> {
        Ok(self
            .inner
            .lock()
            .expect("poisoned")
            .exams
            .iter()
            .find(|e| e.id == id)
            .cloned())
    }

    fn list_templates(&self) -> AppResult<Vec<TemplateSummaryDto>> {
        Ok(self
            .inner
            .lock()
            .expect("poisoned")
            .templates
            .iter()
            .map(|t| TemplateSummaryDto {
                id: t.id,
                title: t.title.clone(),
                created_at: t.created_at.clone(),
            })
            .collect())
    }

    fn get_template(&self, id: i64) -> AppResult<Option<TemplateDto>> {
        Ok(self
            .inner
            .lock()
            .expect("poisoned")
            .templates
            .iter()
            .find(|t| t.id == id)
            .cloned())
    }

    fn list_results(&self, filter: ResultFilter) -> AppResult<Vec<ResultDto>> {
        Ok(self
            .inner
            .lock()
            .expect("poisoned")
            .results
            .iter()
            .filter(|r| filter.exam_id.is_none() || r.exam_id == filter.exam_id)
            .cloned()
            .collect())
    }

    fn create_exam(&self, input: NewExam) -> AppResult<ExamDto> {
        if self.read_only {
            return Err(read_only());
        }
        let mut state = self.inner.lock().expect("poisoned");
        let id = state.bump_id();
        let exam = ExamDto {
            id,
            name: input.name,
            template_id: input.template_id,
            created_at: FIXED_TS.to_string(),
            updated_at: FIXED_TS.to_string(),
        };
        state.exams.push(exam.clone());
        Ok(exam)
    }

    fn create_result(&self, input: NewResult) -> AppResult<ResultDto> {
        if self.read_only {
            return Err(read_only());
        }
        let mut state = self.inner.lock().expect("poisoned");
        let id = state.bump_id();
        let result = ResultDto {
            id,
            student_id: input.student_id,
            template_id: input.template_id,
            exam_id: input.exam_id,
            total_score: input.total_score,
            detail_answers: input.detail_answers,
            image_path: input.image_path,
            created_at: FIXED_TS.to_string(),
        };
        state.results.push(result.clone());
        Ok(result)
    }
}
