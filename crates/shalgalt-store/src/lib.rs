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

use std::path::{Path, PathBuf};
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

    /// Bring the schema up to date, tracking progress in `PRAGMA user_version` (the epoch =
    /// number of applied migrations). Only unapplied steps run, so adding a migration and
    /// restarting the server upgrades an existing server-owned database in place.
    ///
    /// One exception: a database whose `templates` table exists but whose `user_version` is
    /// still 0 is an *externally managed* file — `tauri-plugin-sql` tracks its own
    /// migrations in `_sqlx_migrations` and never sets `user_version`. We leave it
    /// untouched: the desktop registers the same migration files, so the file is already at
    /// the latest schema, and running our DDL over it would fail on `CREATE TABLE`.
    pub fn migrate(&self) -> AppResult<()> {
        let conn = self.lock();

        let has_templates = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'templates'",
                [],
                |_| Ok(()),
            )
            .optional()
            .map_err(to_app)?
            .is_some();
        let user_version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(to_app)?;

        if has_templates && user_version == 0 {
            return Ok(());
        }

        for (i, sql) in MIGRATIONS.iter().enumerate() {
            let version = (i + 1) as i64;
            if user_version < version {
                conn.execute_batch(sql).map_err(to_app)?;
            }
        }
        conn.pragma_update(None, "user_version", MIGRATIONS.len() as i64)
            .map_err(to_app)?;
        Ok(())
    }

    /// Acquire the connection, recovering from a poisoned `Mutex` rather than panicking. A
    /// handler that panicked mid-query would otherwise poison the lock and cascade the panic
    /// into every subsequent request; the connection state itself is fine to reuse for the
    /// next caller.
    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
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

/// A read-only `DataStore` that opens a fresh read-only connection per request and reports
/// an empty database until the file exists.
///
/// The desktop needs this because `tauri-plugin-sql` creates the SQLite file lazily (on the
/// frontend's first `Database.load`), which happens *after* the embedded HTTP server boots.
/// A persistent connection would fail to open at boot; this defers the open to first use
/// and degrades to empty reads while the file is still absent. Writes are always rejected.
/// Traffic is local and low-volume, so per-request connection setup is acceptable.
pub struct DeferredReadOnlyStore {
    path: PathBuf,
}

impl DeferredReadOnlyStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// `None` until the database file exists; afterwards a read-only handle to it.
    ///
    /// The open is attempted first and only on failure is existence checked, so there is no
    /// check-then-open (TOCTOU) window: a genuine "not created yet" reports empty, while a
    /// real open error against an existing file propagates as a 500.
    fn reader(&self) -> AppResult<Option<SqliteStore>> {
        match SqliteStore::open_read_only(&self.path) {
            Ok(store) => Ok(Some(store)),
            Err(_) if !self.path.exists() => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl DataStore for DeferredReadOnlyStore {
    fn list_exams(&self) -> AppResult<Vec<ExamDto>> {
        match self.reader()? {
            Some(s) => s.list_exams(),
            None => Ok(Vec::new()),
        }
    }

    fn get_exam(&self, id: i64) -> AppResult<Option<ExamDto>> {
        match self.reader()? {
            Some(s) => s.get_exam(id),
            None => Ok(None),
        }
    }

    fn list_templates(&self) -> AppResult<Vec<TemplateSummaryDto>> {
        match self.reader()? {
            Some(s) => s.list_templates(),
            None => Ok(Vec::new()),
        }
    }

    fn get_template(&self, id: i64) -> AppResult<Option<TemplateDto>> {
        match self.reader()? {
            Some(s) => s.get_template(id),
            None => Ok(None),
        }
    }

    fn list_results(&self, filter: ResultFilter) -> AppResult<Vec<ResultDto>> {
        match self.reader()? {
            Some(s) => s.list_results(filter),
            None => Ok(Vec::new()),
        }
    }

    fn create_exam(&self, _input: NewExam) -> AppResult<ExamDto> {
        Err(read_only())
    }

    fn create_result(&self, _input: NewResult) -> AppResult<ResultDto> {
        Err(read_only())
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
        // Second migrate sees user_version already at the latest epoch and does nothing.
        store.migrate().unwrap();
        assert!(store.list_exams().unwrap().is_empty());
    }

    #[test]
    fn migrate_applies_a_newly_added_step_to_an_existing_server_db() {
        // Simulate a server DB bootstrapped at an earlier epoch (before 0005 existed):
        // tables present, user_version pinned below MIGRATIONS.len(), exam_id absent.
        let file = NamedTempFile::new().unwrap();
        {
            let conn = rusqlite::Connection::open(file.path()).unwrap();
            for sql in &MIGRATIONS[..4] {
                conn.execute_batch(sql).unwrap();
            }
            conn.pragma_update(None, "user_version", 4).unwrap();
            // The pre-0005 results table has no exam_id column.
            assert!(conn.prepare("SELECT exam_id FROM results").is_err());
        }

        // Restarting with the full MIGRATIONS array must apply only step 5.
        let store = SqliteStore::open_read_write(file.path()).unwrap();
        store.migrate().unwrap();
        // A filtered read now succeeds because results.exam_id exists.
        assert!(store
            .list_results(ResultFilter { exam_id: Some(1) })
            .unwrap()
            .is_empty());
    }

    #[test]
    fn migrate_leaves_externally_managed_db_untouched() {
        // A plugin-sql-style DB: tables exist but user_version is still 0. migrate() must
        // not try to re-run CREATE TABLE over it.
        let file = NamedTempFile::new().unwrap();
        {
            let conn = rusqlite::Connection::open(file.path()).unwrap();
            for sql in MIGRATIONS {
                conn.execute_batch(sql).unwrap();
            }
            // Deliberately leave user_version at 0, as tauri-plugin-sql does.
        }
        let store = SqliteStore::open_read_write(file.path()).unwrap();
        store.migrate().unwrap(); // no-op, no error
        assert!(store.list_templates().unwrap().is_empty());
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
    fn deferred_store_is_empty_until_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.sqlite");
        let store = DeferredReadOnlyStore::new(path.clone());

        // No file yet: reads are empty, writes rejected.
        assert!(store.list_exams().unwrap().is_empty());
        assert!(store.get_exam(1).unwrap().is_none());
        assert!(matches!(
            store
                .create_exam(NewExam {
                    name: "X".into(),
                    template_id: 1,
                })
                .unwrap_err(),
            AppError::BadRequest(_)
        ));

        // Once the file is bootstrapped, the same store reads through to it.
        SqliteStore::open_read_write(&path)
            .unwrap()
            .migrate()
            .unwrap();
        assert!(store.list_exams().unwrap().is_empty());
        assert!(store.get_template(1).unwrap().is_none());
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
