import { invoke } from "@tauri-apps/api/core";
import type { XlsxReport } from "$lib/types/generated/XlsxReport";
import type { ReportLabels } from "$lib/types/generated/ReportLabels";

/**
 * P5-01: render a results report to xlsx and write it to `outputPath`.
 *
 * The caller gathers graded sheets from the DB, shapes them into an
 * `XlsxReport`, and supplies the Mongolian column/sheet labels from the string
 * table. Rule 1: only small JSON + the save path cross the boundary — the
 * workbook bytes are written Rust-side. Returns the absolute path written.
 */
export function exportResultsXlsx(
  report: XlsxReport,
  labels: ReportLabels,
  outputPath: string,
): Promise<string> {
  return invoke("export_results_xlsx", { report, labels, outputPath });
}
