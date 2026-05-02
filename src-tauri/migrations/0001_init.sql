-- Blueprint §4 — initial schema.
-- See docs/ARCHITECTURE.md §6 for design notes.

PRAGMA foreign_keys = ON;

CREATE TABLE students (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT     NOT NULL,
    grade        INTEGER  NOT NULL,
    class        INTEGER  NOT NULL,
    roll_number  TEXT     NOT NULL,
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (grade, class, roll_number)
);

CREATE TABLE templates (
    id           INTEGER  PRIMARY KEY AUTOINCREMENT,
    title        TEXT     NOT NULL,
    json_schema  TEXT     NOT NULL,                       -- Rule 3: serialized OmrTemplate.
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE results (
    id              INTEGER  PRIMARY KEY AUTOINCREMENT,
    student_id      INTEGER  REFERENCES students(id) ON DELETE SET NULL,
    template_id     INTEGER  NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
    total_score     REAL     NOT NULL,
    detail_answers  TEXT     NOT NULL,                    -- JSON: per-question correct/wrong/blank.
    image_path      TEXT,                                 -- Absolute path to the result image (Rule 1).
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_results_template ON results(template_id);
CREATE INDEX idx_results_student  ON results(student_id);
