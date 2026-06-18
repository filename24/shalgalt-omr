/**
 * Orchestrates the xlsx export for a completed grading job (P5-02): build the
 * payload, ask for a save path, write it Rust-side, and surface the result.
 *
 * The pure shaping lives in `./xlsxReport.ts`; this module owns the Tauri
 * surface (save dialog + IPC) and is intentionally thin.
 */
import { toast } from "svelte-sonner";

import { mn } from "$lib/i18n";
import { exportResultsXlsx } from "$lib/ipc/export";
import { pickXlsxSavePath } from "$lib/picker";
import { getJobById, parseGradedSheets } from "$lib/db/jobs";
import { getTemplate } from "$lib/db/templates";
import type { OmrTemplate } from "$lib/types/template";
import type { AnswerKey } from "$lib/types/generated/AnswerKey";
import type { GradedJobSheet } from "$lib/types/job";
import { buildReportLabels, buildXlsxReport } from "./xlsxReport";
import {
  mergeAnswerKeysForColumns,
  parseStoredAnswerKeys,
} from "./storedAnswerKeys";

/** Strip characters that are awkward in file names; keep it simple and safe. */
function safeFileName(title: string): string {
  const cleaned = title.replace(/[\\/:*?"<>|]+/g, "_").trim();
  return (cleaned.length > 0 ? cleaned : "results") + ".xlsx";
}

export interface ExportJobInput {
  title: string;
  template: OmrTemplate;
  answerKey: AnswerKey;
  sheets: GradedJobSheet[];
}

/**
 * Run the full export flow. Returns the written path, or `null` when the user
 * cancels the save dialog. Errors are surfaced via toast and rethrown so callers
 * can react (e.g. clear a busy flag).
 */
export async function exportJobToXlsx(
  input: ExportJobInput,
): Promise<string | null> {
  const report = buildXlsxReport({
    title: input.title,
    template: input.template,
    answerKey: input.answerKey,
    sheets: input.sheets,
    studentFallback: mn.results.export.studentFallback,
  });
  const labels = buildReportLabels();

  const path = await pickXlsxSavePath(safeFileName(input.title));
  if (!path) return null;

  try {
    const written = await exportResultsXlsx(report, labels, path);
    toast.success(mn.results.export.success);
    return written;
  } catch (e) {
    toast.error(mn.results.export.failed, { description: String(e) });
    throw e;
  }
}

/**
 * Load everything a job needs and run the export. Used by both the results list
 * (lazy — only fetches the blob when the user clicks Export) and the detail
 * page. Returns the written path, or `null` when the job is unusable or the user
 * cancels the dialog.
 */
export async function exportJobById(jobId: number): Promise<string | null> {
  const job = await getJobById(jobId);
  if (!job || job.status !== "done") {
    toast.error(mn.results.detail.notFinished);
    return null;
  }
  const template = await getTemplate(job.template_id);
  if (!template) {
    toast.error(mn.results.export.failed, { description: mn.review.notFound });
    return null;
  }
  let answerKey: AnswerKey;
  try {
    // The job stores one key per variant; the xlsx columns come from the union
    // of their question groups.
    answerKey = mergeAnswerKeysForColumns(
      parseStoredAnswerKeys(job.answer_key_json),
    );
  } catch (e) {
    toast.error(mn.results.export.failed, { description: String(e) });
    return null;
  }
  return exportJobToXlsx({
    title: template.schema.title,
    template: template.schema,
    answerKey,
    sheets: parseGradedSheets(job),
  });
}
