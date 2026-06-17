/**
 * Runtime guard for the `AnswerKey` JSON shape. Mirrors
 * `shalgalt_core::domain::answer_key::AnswerKey` so we surface schema mistakes
 * (typos, missing `correct_indices`, off-by-one `exam_id` types) before the
 * Rust round-trip in `scan_grade_pdf`.
 *
 * `/grade` now selects a stored key via the exam/variant picker (P4-04), so the
 * paste-based flow this once guarded is gone. The remaining consumers are the
 * `.shalgalt` import path (`$lib/project`) and the `exams` answer-key editors,
 * where `answerKeyEntrySchema` validates untrusted external/edited input.
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
