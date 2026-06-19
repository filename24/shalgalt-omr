-- P3-08 — durable batch-grading jobs.
--
-- Each row tracks a single `scan_grade_pdf` invocation: the source PDF, the
-- template + answer key it was graded against, the live progress counters, and
-- (once the job terminates) the per-page graded sheets serialized as JSON.
--
-- `answer_key_json` is a free-form `AnswerKey` payload provided by the user
-- via the `/grade` page (paste or pick `.json`). When P4-04 lands the proper
-- `exams` / `answer_keys` schema, this column becomes redundant and a follow-up
-- migration can drop it in favour of an `answer_key_id` FK.
--
-- `graded_sheets_json` is Rule 3 territory: a JSON array of `GradedJobSheet`
-- objects (`{ page_index, page_image_path, parsed, graded, reviewed }`).
-- Per-sheet rows are not needed because `/review` only ever loads one job at a
-- time — same precedent as `templates.json_schema`.

PRAGMA foreign_keys = ON;

CREATE TABLE jobs (
    id                  INTEGER  PRIMARY KEY AUTOINCREMENT,
    task_id             TEXT     NOT NULL UNIQUE,
    pdf_path            TEXT     NOT NULL,
    template_id         INTEGER  NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
    answer_key_json     TEXT     NOT NULL,
    status              TEXT     NOT NULL CHECK (status IN ('queued','running','done','failed','canceled')),
    total_pages         INTEGER,
    processed_pages     INTEGER  NOT NULL DEFAULT 0,
    needs_review_count  INTEGER  NOT NULL DEFAULT 0,
    graded_sheets_json  TEXT,
    error_message       TEXT,
    created_at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_jobs_task_id ON jobs(task_id);
CREATE INDEX idx_jobs_status_created ON jobs(status, created_at DESC);

CREATE TRIGGER trg_jobs_updated_at
AFTER UPDATE ON jobs
FOR EACH ROW
BEGIN
    UPDATE jobs SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
