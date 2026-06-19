/**
 * Pre-grade validation: does the selected answer key actually target any of the
 * selected template's question groups?
 *
 * The grading engine treats a `question` group with no matching answer-key entry
 * as "not part of this exam" and skips it entirely (partial-exam support — see
 * `crates/shalgalt-core/src/grading/engine.rs`). That is intentional, but it has
 * a silent-failure edge: if the key matches *zero* question groups (e.g. the
 * template was edited and its group ids changed after the key was authored, or
 * the sheets were printed from a different template), every question is skipped
 * and the results come back empty with no indication why.
 *
 * This module surfaces that case at grade-start time so the teacher gets an
 * explicit warning instead of an empty results screen.
 */
import type { OmrTemplate } from "$lib/types/template";
import type { AnswerKey } from "$lib/types/generated/AnswerKey";

/**
 * Count the template's `question` groups that the answer key actually targets.
 * Mirrors the engine's `AnswerKey::correct_for` lookup: a question counts as
 * "in this exam" exactly when an entry's `group_id` equals the group's `id`.
 *
 * Returns the number of matched question groups; `0` means the key matches
 * nothing and grading would drop every question.
 */
export function countMatchedQuestionGroups(
  template: OmrTemplate,
  answerKey: AnswerKey,
): number {
  const keyedGroupIds = new Set(answerKey.answers.map((entry) => entry.group_id));
  return template.groups.filter(
    (group) => group.kind === "question" && keyedGroupIds.has(group.id),
  ).length;
}

/**
 * `true` when the answer key targets at least one of the template's question
 * groups. `false` means grading would skip every question and produce an empty
 * result — the caller should warn and abort before starting the job.
 */
export function answerKeyMatchesTemplate(
  template: OmrTemplate,
  answerKey: AnswerKey,
): boolean {
  return countMatchedQuestionGroups(template, answerKey) > 0;
}
