-- P4-04 — exams + per-variant answer keys.
--
-- An exam binds a name to a template; each exam has one or more variants
-- (A/B/C/...), and each variant carries one answer key. The AnswerKey domain
-- type (shalgalt_core::domain::answer_key) is { exam_id, variant, answers };
-- the DB stores exam_id + variant on the row and the answers array as JSON.
-- UNIQUE (exam_id, variant) enforces the domain invariant.
--
-- jobs.answer_key_json (migration 0003) stays as-is for now; rewiring /grade to
-- consume these tables is a separate follow-up (P4-05/06).
PRAGMA foreign_keys = ON;

CREATE TABLE exams (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT     NOT NULL,
    template_id  INTEGER  NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE answer_keys (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    exam_id      INTEGER  NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    variant      TEXT     NOT NULL,
    answers_json TEXT     NOT NULL,  -- JSON array of AnswerKeyEntry
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (exam_id, variant)
);

CREATE INDEX idx_exams_template ON exams(template_id);
CREATE INDEX idx_answer_keys_exam ON answer_keys(exam_id);

CREATE TRIGGER trg_exams_updated_at
AFTER UPDATE ON exams FOR EACH ROW
BEGIN UPDATE exams SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id; END;

CREATE TRIGGER trg_answer_keys_updated_at
AFTER UPDATE ON answer_keys FOR EACH ROW
BEGIN UPDATE answer_keys SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id; END;
