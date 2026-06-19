/**
 * Normalize the `jobs.answer_key_json` column into the shapes the results
 * consumers need.
 *
 * History: pre-auto-detect jobs stored a single serialized `AnswerKey`.
 * Mixed-variant grading stores an `AnswerKey[]` (one per variant) so each sheet
 * can be graded against its own variant. Both forms live in the same TEXT
 * column, so every reader normalizes through here instead of assuming one shape.
 */
import type { AnswerKey } from "$lib/types/generated/AnswerKey";
import type { AnswerKeyEntry } from "$lib/types/generated/AnswerKeyEntry";

/**
 * Parse the stored column into a list of answer keys. Accepts both the legacy
 * single-object form and the array form.
 */
export function parseStoredAnswerKeys(json: string): AnswerKey[] {
  const parsed = JSON.parse(json) as unknown;
  return Array.isArray(parsed)
    ? (parsed as AnswerKey[])
    : [parsed as AnswerKey];
}

/**
 * Collapse every variant's keys into one synthetic key whose `answers` is the
 * union of question groups (first-seen order). The xlsx report uses this only to
 * derive the question *columns* (group_id + label); correct indices are
 * irrelevant here, and variants of one exam share the same template question
 * set, so the union is just defensive against partial/uneven keys.
 */
export function mergeAnswerKeysForColumns(keys: AnswerKey[]): AnswerKey {
  const seen = new Set<string>();
  const answers: AnswerKeyEntry[] = [];
  for (const key of keys) {
    for (const entry of key.answers) {
      if (!seen.has(entry.group_id)) {
        seen.add(entry.group_id);
        answers.push(entry);
      }
    }
  }
  return { exam_id: keys[0]?.exam_id ?? 0, variant: "*", answers };
}
