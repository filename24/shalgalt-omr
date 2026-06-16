/**
 * Frontend types for the `exams` and `answer_keys` tables (P4-04).
 *
 * Like `job.ts`, the DB row shapes (`Exam`, `AnswerKeyRecord`) are hand-written
 * because `tauri-plugin-sql` owns the schema and ts-rs only mirrors
 * `shalgalt-core` domain types. The answer payload reuses the generated
 * `AnswerKeyEntry` / `AnswerKey` from the Rust core so the round-trip into
 * grading stays type-aligned.
 */
import { z } from "zod";
import type { AnswerKeyEntry } from "$lib/types/generated/AnswerKeyEntry";
import type { AnswerKey } from "$lib/types/generated/AnswerKey";
import { answerKeyEntrySchema } from "$lib/schemas/answerKey";

export interface Exam {
  id: number;
  name: string;
  template_id: number;
  created_at: string;
  updated_at: string;
}

/**
 * An exam row enriched for list views: the joined template title and the count
 * of answer keys (variants) attached to the exam.
 */
export interface ExamSummary extends Exam {
  template_title: string;
  variant_count: number;
}

/**
 * One answer-key row. `answers` is the decoded `answers_json` column; together
 * with `exam_id` + `variant` it reconstructs the `AnswerKey` domain type via
 * `toAnswerKey(...)`.
 */
export interface AnswerKeyRecord {
  id: number;
  exam_id: number;
  variant: string;
  answers: AnswerKeyEntry[];
  created_at: string;
  updated_at: string;
}

export const examNameSchema = z.string().min(1);

export const answersSchema = z.array(answerKeyEntrySchema).min(1);

/**
 * Project an `AnswerKeyRecord` (DB shape) down to the `AnswerKey` domain type
 * the grading core consumes.
 */
export function toAnswerKey(rec: AnswerKeyRecord): AnswerKey {
  return { exam_id: rec.exam_id, variant: rec.variant, answers: rec.answers };
}
