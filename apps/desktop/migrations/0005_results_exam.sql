-- P5-04 — results gain an exam_id so the REST API can filter graded results by exam
-- (`GET /v1/results?exam_id=...`) and the write endpoint (`POST /v1/results`) can link a
-- posted result to its exam.
--
-- Nullable + `ON DELETE SET NULL`: legacy rows and desktop job-results that predate exam
-- linkage keep working, and deleting an exam detaches its results rather than cascading
-- them away. The same migration is applied by `shalgalt-store` when `apps/server`
-- bootstraps a fresh database, so the column exists regardless of which host created it.
PRAGMA foreign_keys = ON;

ALTER TABLE results ADD COLUMN exam_id INTEGER REFERENCES exams(id) ON DELETE SET NULL;

CREATE INDEX idx_results_exam ON results(exam_id);
