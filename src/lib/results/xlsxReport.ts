/**
 * Pure shaping of completed grading jobs into the xlsx export payload (P5-02).
 *
 * Kept side-effect free so it is unit-testable: no Tauri, no dialog, no DB. The
 * orchestration (save dialog + IPC + toasts) lives in `./export.ts`.
 */
import type { OmrTemplate } from "$lib/types/template";
import type { AnswerKey } from "$lib/types/generated/AnswerKey";
import type { GradedJobSheet } from "$lib/types/job";
import type { XlsxReport } from "$lib/types/generated/XlsxReport";
import type { ReportLabels } from "$lib/types/generated/ReportLabels";
import { mn } from "$lib/i18n";

export interface BuildReportInput {
  /** Workbook title row — typically the template/exam name. */
  title: string;
  template: OmrTemplate;
  /** The exam's question set: only keyed groups become columns. */
  answerKey: AnswerKey;
  sheets: GradedJobSheet[];
  /** Prefix for sheets with no bubbled student id, e.g. "Сурагч". */
  studentFallback: string;
}

/** Resolve a `BubbleGroup.id` to its human label, falling back to the id. */
function labelFor(template: OmrTemplate, groupId: string): string {
  const group = template.groups.find((g) => g.id === groupId);
  return group?.label?.trim() ? group.label : groupId;
}

/** A sheet's display name: bubbled student id, else a stable fallback. */
function studentLabel(
  sheet: GradedJobSheet,
  index: number,
  fallback: string,
): string {
  const text = sheet.parsed.student_id_text?.trim();
  return text ? text : `${fallback} ${index + 1}`;
}

/**
 * Build the {@link XlsxReport} the Rust writer consumes. Question columns follow
 * the answer key's order (the exam's question set), so unkeyed template groups
 * never leak into the export — matching the grading engine's "answer key is the
 * question set" rule.
 */
export function buildXlsxReport(input: BuildReportInput): XlsxReport {
  const questions = input.answerKey.answers.map((entry) => ({
    group_id: entry.group_id,
    label: labelFor(input.template, entry.group_id),
  }));

  const rows = input.sheets.map((sheet, i) => ({
    label: studentLabel(sheet, i, input.studentFallback),
    total_score: sheet.graded.total_score,
    needs_review: sheet.graded.needs_review,
    answers: sheet.graded.answers,
  }));

  return { title: input.title, questions, rows };
}

/**
 * Project the Mongolian string table into the {@link ReportLabels} the writer
 * needs. Keeping every workbook string here means the Rust core stays
 * English-only (AGENTS.md language rule).
 */
export function buildReportLabels(): ReportLabels {
  const e = mn.results.export;
  return {
    summary_sheet: e.sheetSummary,
    per_question_sheet: e.sheetPerQuestion,
    errors_sheet: e.sheetErrors,
    col_index: e.colIndex,
    col_student: e.colStudent,
    col_score: e.colScore,
    col_status: e.colStatus,
    col_question: e.colQuestion,
    col_correct: e.colCorrect,
    col_wrong: e.colWrong,
    col_blank: e.colBlank,
    col_partial: e.colPartial,
    col_uncertain: e.colUncertain,
    col_accuracy: e.colAccuracy,
    col_outcome: e.colOutcome,
    status_ok: e.statusOk,
    status_needs_review: e.statusNeedsReview,
    outcome_wrong: e.outcomeWrong,
    outcome_blank: e.outcomeBlank,
    outcome_multiple: e.outcomeMultiple,
    outcome_partial: e.outcomePartial,
    outcome_uncertain: e.outcomeUncertain,
    empty: e.empty,
  };
}
