/**
 * Repository for the `answer_keys` table (P4-04).
 *
 * Same pattern as `jobs.ts`: every call goes through `getDb()`, queries are
 * parameterized, and the JSON-shaped `answers_json` column is parsed once at
 * the boundary so callers see typed `AnswerKeyEntry[]`, never raw text.
 *
 * Each row is one variant's answer key for an exam. `UNIQUE (exam_id, variant)`
 * enforces the domain invariant; `upsertAnswerKey(...)` relies on it for the
 * INSERT ... ON CONFLICT path.
 */
import { getDb } from "./index";
import type { AnswerKeyRecord } from "$lib/types/exam";
import type { AnswerKeyEntry } from "$lib/types/generated/AnswerKeyEntry";

interface AnswerKeyRow {
  id: number;
  exam_id: number;
  variant: string;
  answers_json: string;
  created_at: string;
  updated_at: string;
}

const SELECT_COLUMNS = `id, exam_id, variant, answers_json, created_at, updated_at`;

function rowToRecord(r: AnswerKeyRow): AnswerKeyRecord {
  return {
    id: r.id,
    exam_id: r.exam_id,
    variant: r.variant,
    answers: JSON.parse(r.answers_json) as AnswerKeyEntry[],
    created_at: r.created_at,
    updated_at: r.updated_at,
  };
}

export interface NewAnswerKeyInput {
  exam_id: number;
  variant: string;
  answers: AnswerKeyEntry[];
}

export async function createAnswerKey(
  input: NewAnswerKeyInput,
): Promise<AnswerKeyRecord> {
  const db = await getDb();
  const result = await db.execute(
    `INSERT INTO answer_keys (exam_id, variant, answers_json)
     VALUES ($1, $2, $3)`,
    [input.exam_id, input.variant, JSON.stringify(input.answers)],
  );
  const id = Number(result.lastInsertId);
  const rows = await db.select<AnswerKeyRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM answer_keys WHERE id = $1`,
    [id],
  );
  if (rows.length === 0) {
    throw new Error(`createAnswerKey: row ${id} disappeared after insert`);
  }
  return rowToRecord(rows[0]!);
}

export async function listAnswerKeysByExam(
  examId: number,
): Promise<AnswerKeyRecord[]> {
  const db = await getDb();
  const rows = await db.select<AnswerKeyRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM answer_keys WHERE exam_id = $1 ORDER BY variant`,
    [examId],
  );
  return rows.map(rowToRecord);
}

export async function getAnswerKey(
  examId: number,
  variant: string,
): Promise<AnswerKeyRecord | null> {
  const db = await getDb();
  const rows = await db.select<AnswerKeyRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM answer_keys WHERE exam_id = $1 AND variant = $2`,
    [examId, variant],
  );
  return rows.length > 0 ? rowToRecord(rows[0]!) : null;
}

/**
 * Insert or replace the answer key for `(exam_id, variant)`. Relies on the
 * `UNIQUE (exam_id, variant)` constraint to resolve the conflict, overwriting
 * `answers_json` on an existing row. Re-selects the resulting row so callers
 * always get the canonical record back.
 */
export async function upsertAnswerKey(
  input: NewAnswerKeyInput,
): Promise<AnswerKeyRecord> {
  const db = await getDb();
  await db.execute(
    `INSERT INTO answer_keys (exam_id, variant, answers_json)
     VALUES ($1, $2, $3)
     ON CONFLICT(exam_id, variant)
     DO UPDATE SET answers_json = excluded.answers_json`,
    [input.exam_id, input.variant, JSON.stringify(input.answers)],
  );
  const rows = await db.select<AnswerKeyRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM answer_keys WHERE exam_id = $1 AND variant = $2`,
    [input.exam_id, input.variant],
  );
  if (rows.length === 0) {
    throw new Error(
      `upsertAnswerKey: row for exam ${input.exam_id} variant ${input.variant} disappeared after upsert`,
    );
  }
  return rowToRecord(rows[0]!);
}

export interface AnswerKeyPatch {
  variant?: string;
  answers?: AnswerKeyEntry[];
}

export async function updateAnswerKey(
  id: number,
  patch: AnswerKeyPatch,
): Promise<void> {
  const db = await getDb();
  if (patch.variant === undefined && patch.answers === undefined) {
    return;
  }
  const sets: string[] = [];
  const params: (string | number)[] = [];
  let i = 1;
  if (patch.variant !== undefined) {
    sets.push(`variant = $${i++}`);
    params.push(patch.variant);
  }
  if (patch.answers !== undefined) {
    sets.push(`answers_json = $${i++}`);
    params.push(JSON.stringify(patch.answers));
  }
  params.push(id);
  await db.execute(
    `UPDATE answer_keys SET ${sets.join(", ")} WHERE id = $${i}`,
    params,
  );
}

export async function deleteAnswerKey(id: number): Promise<void> {
  const db = await getDb();
  await db.execute(`DELETE FROM answer_keys WHERE id = $1`, [id]);
}
