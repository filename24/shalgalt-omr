/**
 * Repository for the `results` table (frontend).
 */
import { getDb } from "./index";
import type { ResultSummary } from "$lib/types/result";

interface ResultRow {
  id: number;
  student_id: number | null;
  template_id: number;
  total_score: number;
  detail_answers: string;
  image_path: string | null;
}

function rowToSummary(r: ResultRow): ResultSummary {
  return {
    id: r.id,
    student_id: r.student_id,
    template_id: r.template_id,
    total_score: r.total_score,
    detail_answers_json: r.detail_answers,
    image_path: r.image_path,
  };
}

export async function listResultsByTemplate(
  templateId: number,
): Promise<ResultSummary[]> {
  const db = await getDb();
  const rows = await db.select<ResultRow[]>(
    `SELECT id, student_id, template_id, total_score, detail_answers, image_path
     FROM results WHERE template_id = $1 ORDER BY id DESC`,
    [templateId],
  );
  return rows.map(rowToSummary);
}

export interface NewResult {
  student_id: number | null;
  template_id: number;
  total_score: number;
  detail_answers_json: string;
  image_path: string | null;
}

export async function insertResult(input: NewResult): Promise<number> {
  const db = await getDb();
  const result = await db.execute(
    `INSERT INTO results (student_id, template_id, total_score, detail_answers, image_path)
     VALUES ($1, $2, $3, $4, $5)`,
    [
      input.student_id,
      input.template_id,
      input.total_score,
      input.detail_answers_json,
      input.image_path,
    ],
  );
  return Number(result.lastInsertId);
}
