//! `SqliteStore` — a `rusqlite` implementation of [`shalgalt_core::api::store::DataStore`].
//!
//! This is the one place a second SQLite connection (besides `tauri-plugin-sql`'s sqlx
//! pool) is allowed. Two postures, picked by the host:
//!
//! - [`SqliteStore::open_read_only`] — `apps/desktop`. Opens the plugin-sql database file
//!   `SQLITE_OPEN_READ_ONLY`, so the embedded HTTP API can read but never compete with the
//!   plugin-sql writer (which remains the single source of truth for writes). `create_*`
//!   is rejected with the core read-only error.
//! - [`SqliteStore::open_read_write`] + [`SqliteStore::migrate`] — `apps/server`. Owns the
//!   whole file; bootstraps the schema for a fresh database and serves the full read/write
//!   surface.
//!
//! The schema is the desktop's migration SQL, embedded verbatim so the server applies the
//! identical DDL `tauri-plugin-sql` runs — there is exactly one source of truth for the
//! schema (`apps/desktop/migrations/`).

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Row};

use shalgalt_core::api::dto::{
    ExamDto, NewExam, NewResult, ResultDto, ResultFilter, TemplateDto, TemplateSummaryDto,
};
use shalgalt_core::api::store::{read_only, DataStore};
use shalgalt_core::error::{AppError, AppResult};

/// The desktop's migration files, in version order. Referenced by path (not copied) so the
/// server cannot drift from the schema `tauri-plugin-sql` applies on the desktop side.
const MIGRATIONS: &[&str] = &[
    include_str!("../../../apps/desktop/migrations/0001_init.sql"),
    include_str!("../../../apps/desktop/migrations/0002_backdrop.sql"),
    include_str!("../../../apps/desktop/migrations/0003_jobs.sql"),
    include_str!("../../../apps/desktop/migrations/0004_exams.sql"),
    include_str!("../../../apps/desktop/migrations/0005_results_exam.sql"),
];

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// rusqlite errors are internal faults from the API's perspective — the stable `code` the
/// client sees is `internal`; the detail goes to logs only.
fn to_app(err: rusqlite::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!(err))
}

/// A `DataStore` backed by a single SQLite connection guarded by a `Mutex` (rusqlite's
/// `Connection` is `Send` but not `Sync`).
pub struct SqliteStore {
    conn: Mutex<Connection>,
    read_only: bool,
}

impl SqliteStore {
    /// Open the database read-only. Suitable for the desktop, where `tauri-plugin-sql`
    /// owns writes. `create_*` calls are rejected.
    pub fn open_read_only(path: &Path) -> AppResult<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(to_app)?;
        conn.busy_timeout(BUSY_TIMEOUT).map_err(to_app)?;
        Ok(Self {
            conn: Mutex::new(conn),
            read_only: true,
        })
    }

    /// Open (creating if absent) the database read-write. Used by the standalone server,
    /// which owns the file. Call [`migrate`](Self::migrate) afterwards to bootstrap the
    /// schema.
    pub fn open_read_write(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path).map_err(to_app)?;
        conn.busy_timeout(BUSY_TIMEOUT).map_err(to_app)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(to_app)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(to_app)?;
        Ok(Self {
            conn: Mutex::new(conn),
            read_only: false,
        })
    }

    /// Bootstrap the schema for a freshly created database. No-op when the core tables
    /// already exist (a desktop-managed file, or a prior server run) — schema evolution on
    /// an existing file stays owned by the desktop's plugin-sql migrations, never this
    /// path.
    pub fn migrate(&self) -> AppResult<()> {
        let conn = self.conn.lock().expect("store poisoned");
        let already: bool = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'templates'",
                [],
                |_| Ok(true),
            )
            .optional()
            .map_err(to_app)?
            .unwrap_or(false);
        if already {
            return Ok(());
        }
        for sql in MIGRATIONS {
            conn.execute_batch(sql).map_err(to_app)?;
        }
        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("store poisoned")
    }
}

fn exam_from_row(row: &Row<'_>) -> rusqlite::Result<ExamDto> {
    Ok(ExamDto {
        id: row.get(0)?,
        name: row.get(1)?,
        template_id: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn result_from_row(row: &Row<'_>) -> rusqlite::Result<ResultDto> {
    Ok(ResultDto {
        id: row.get(0)?,
        student_id: row.get(1)?,
        template_id: row.get(2)?,
        exam_id: row.get(3)?,
        total_score: row.get(4)?,
        detail_answers: row.get(5)?,
        image_path: row.get(6)?,
        created_at: row.get(7)?,
    })
}

const EXAM_COLS: &str = "id, name, template_id, created_at, updated_at";
const RESULT_COLS: &str =
    "id, student_id, template_id, exam_id, total_score, detail_answers, image_path, created_at";

impl DataStore for SqliteStore {
    fn list_exams(&self) -> AppResult<Vec<ExamDto>> {
        let conn = self.lock();
        let mut stmt = conn
            .prepare(&format!("SELECT {EXAM_COLS} FROM exams ORDER BY id"))
            .map_err(to_app)?;
        let rows = stmt
            .query_map([], exam_from_row)
            .map_err(to_app)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(to_app)?;
        Ok(rows)
    }

    fn get_exam(&self, id: i64) -> AppResult<Option<ExamDto>> {
        let conn = self.lock();
        conn.query_row(
            &format!("SELECT {EXAM_COLS} FROM exams WHERE id = ?1"),
            params![id],
            exam_from_row,
        )
        .optional()
        .map_err(to_app)
    }

    fn list_templates(&self) -> AppResult<Vec<TemplateSummaryDto>> {
        let conn = self.lock();
        let mut stmt = conn
            .prepare("SELECT id, title, created_at FROM templates ORDER BY id")
            .map_err(to_app)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(TemplateSummaryDto {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(to_app)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(to_app)?;
        Ok(rows)
    }

    fn get_template(&self, id: i64) -> AppResult<Option<TemplateDto>> {
        let conn = self.lock();
        conn.query_row(
            "SELECT id, title, json_schema, created_at FROM templates WHERE id = ?1",
            params![id],
            |row| {
                Ok(TemplateDto {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    json_schema: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(to_app)
    }

    fn list_results(&self, filter: ResultFilter) -> AppResult<Vec<ResultDto>> {
        let conn = self.lock();
        let rows = match filter.exam_id {
            Some(exam_id) => {
                let mut stmt = conn
                    .prepare(&format!(
                        "SELECT {RESULT_COLS} FROM results WHERE exam_id = ?1 ORDER BY id"
                    ))
                    .map_err(to_app)?;
                let out = stmt
                    .query_map(params![exam_id], result_from_row)
                    .map_err(to_app)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_app)?;
                out
            }
            None => {
                let mut stmt = conn
                    .prepare(&format!("SELECT {RESULT_COLS} FROM results ORDER BY id"))
                    .map_err(to_app)?;
                let out = stmt
                    .query_map([], result_from_row)
                    .map_err(to_app)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_app)?;
                out
            }
        };
        Ok(rows)
    }

    fn create_exam(&self, input: NewExam) -> AppResult<ExamDto> {
        if self.read_only {
            return Err(read_only());
        }
        let conn = self.lock();
        conn.execute(
            "INSERT INTO exams (name, template_id) VALUES (?1, ?2)",
            params![input.name, input.template_id],
        )
        .map_err(to_app)?;
        let id = conn.last_insert_rowid();
        conn.query_row(
            &format!("SELECT {EXAM_COLS} FROM exams WHERE id = ?1"),
            params![id],
            exam_from_row,
        )
        .map_err(to_app)
    }

    fn create_result(&self, input: NewResult) -> AppResult<ResultDto> {
        if self.read_only {
            return Err(read_only());
        }
        let conn = self.lock();
        conn.execute(
            "INSERT INTO results \
                (student_id, template_id, exam_id, total_score, detail_answers, image_path) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                input.student_id,
                input.template_id,
                input.exam_id,
                input.total_score,
                input.detail_answers,
                input.image_path,
            ],
        )
        .map_err(to_app)?;
        let id = conn.last_insert_rowid();
        conn.query_row(
            &format!("SELECT {RESULT_COLS} FROM results WHERE id = ?1"),
            params![id],
            result_from_row,
        )
        .map_err(to_app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn fresh_store() -> (NamedTempFile, SqliteStore) {
        let file = NamedTempFile::new().unwrap();
        let store = SqliteStore::open_read_write(file.path()).unwrap();
        store.migrate().unwrap();
        (file, store)
    }

    #[test]
    fn migrate_is_idempotent() {
        let (_f, store) = fresh_store();
        // Second migrate sees the tables and does nothing.
        store.migrate().unwrap();
        assert!(store.list_exams().unwrap().is_empty());
    }

    #[test]
    fn create_and_filter_results_by_exam() {
        let (_f, store) = fresh_store();
        store
            .lock()
            .execute(
                "INSERT INTO templates (id, title, json_schema) VALUES (1, 'T', '{}')",
                [],
            )
            .unwrap();
        let exam = store
            .create_exam(NewExam {
                name: "Final".into(),
                template_id: 1,
            })
            .unwrap();

        store
            .create_result(NewResult {
                student_id: None,
                template_id: 1,
                exam_id: Some(exam.id),
                total_score: 12.5,
                detail_answers: "[]".into(),
                image_path: None,
            })
            .unwrap();
        store
            .create_result(NewResult {
                student_id: None,
                template_id: 1,
                exam_id: None,
                total_score: 3.0,
                detail_answers: "[]".into(),
                image_path: None,
            })
            .unwrap();

        let all = store.list_results(ResultFilter::default()).unwrap();
        assert_eq!(all.len(), 2);

        let filtered = store
            .list_results(ResultFilter {
                exam_id: Some(exam.id),
            })
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].total_score, 12.5);
    }

    #[test]
    fn read_only_store_rejects_writes() {
        let file = NamedTempFile::new().unwrap();
        // Bootstrap via a writable handle, then reopen read-only.
        SqliteStore::open_read_write(file.path())
            .unwrap()
            .migrate()
            .unwrap();
        let ro = SqliteStore::open_read_only(file.path()).unwrap();
        let err = ro
            .create_exam(NewExam {
                name: "X".into(),
                template_id: 1,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
