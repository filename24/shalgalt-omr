/**
 * Runtime guard for the `AnswerKey` JSON the user pastes (or picks) on
 * `/grade`. Mirrors `shalgalt_core::domain::answer_key::AnswerKey` so we
 * surface schema mistakes (typos, missing `correct_indices`, off-by-one
 * `exam_id` types) before the Rust round-trip in `scan_grade_pdf`.
 *
 * P4-04 will replace this paste-based flow with a real exam picker; until
 * then, this schema is the contract `/grade` enforces.
 */
import { z } from "zod";

export const answerKeyEntrySchema = z.object({
  group_id: z.string().min(1),
  correct_indices: z.array(z.number().int().min(0)).min(1),
});

export const answerKeySchema = z.object({
  exam_id: z.number().int(),
  variant: z.string().min(1),
  answers: z.array(answerKeyEntrySchema).min(1),
});

export type AnswerKeyInput = z.infer<typeof answerKeySchema>;
