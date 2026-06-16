/**
 * Repository for the `exams` table (P4-04).
 *
 * Same pattern as `jobs.ts` / `templates.ts`: every call goes through
 * `getDb()`, queries are parameterized, and the re-select-after-insert idiom
 * returns the freshly written row so callers see server-side defaults
 * (`id`, `created_at`, `updated_at`) without a second round-trip.
 *
 * An exam binds a name to a template. Per-variant answer keys live in the
 * sibling `answer_keys` table (`answerKeys.ts`); `listExams(...)` surfaces the
 * variant count and the joined template title for list views.
 */
import { getDb } from "./index";
import type { Exam, ExamSummary } from "$lib/types/exam";

interface ExamRow {
  id: number;
  name: string;
  template_id: number;
  created_at: string;
  updated_at: string;
}

interface ExamSummaryRow extends ExamRow {
  template_title: string;
  variant_count: number;
}

const SELECT_COLUMNS = `id, name, template_id, created_at, updated_at`;

function rowToExam(r: ExamRow): Exam {
  return {
    id: r.id,
    name: r.name,
    template_id: r.template_id,
    created_at: r.created_at,
    updated_at: r.updated_at,
  };
}

function rowToExamSummary(r: ExamSummaryRow): ExamSummary {
  return {
    ...rowToExam(r),
    template_title: r.template_title,
    variant_count: r.variant_count,
  };
}

export interface NewExamInput {
  name: string;
  template_id: number;
}

export async function createExam(input: NewExamInput): Promise<Exam> {
  const db = await getDb();
  const result = await db.execute(
    `INSERT INTO exams (name, template_id) VALUES ($1, $2)`,
    [input.name, input.template_id],
  );
  const id = Number(result.lastInsertId);
  const rows = await db.select<ExamRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM exams WHERE id = $1`,
    [id],
  );
  if (rows.length === 0) {
    throw new Error(`createExam: row ${id} disappeared after insert`);
  }
  return rowToExam(rows[0]!);
}

export async function listExams(): Promise<ExamSummary[]> {
  const db = await getDb();
  const rows = await db.select<ExamSummaryRow[]>(
    `SELECT e.id, e.name, e.template_id, e.created_at, e.updated_at,
            COALESCE(t.title, '') AS template_title,
            (SELECT COUNT(*) FROM answer_keys WHERE exam_id = e.id) AS variant_count
     FROM exams e
     LEFT JOIN templates t ON t.id = e.template_id
     ORDER BY e.created_at DESC, e.id DESC`,
  );
  return rows.map(rowToExamSummary);
}

export async function getExamById(id: number): Promise<Exam | null> {
  const db = await getDb();
  const rows = await db.select<ExamRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM exams WHERE id = $1`,
    [id],
  );
  return rows.length > 0 ? rowToExam(rows[0]!) : null;
}

export async function updateExam(
  id: number,
  patch: { name: string },
): Promise<void> {
  const db = await getDb();
  await db.execute(`UPDATE exams SET name = $1 WHERE id = $2`, [patch.name, id]);
}

export async function deleteExam(id: number): Promise<void> {
  const db = await getDb();
  await db.execute(`DELETE FROM exams WHERE id = $1`, [id]);
}
