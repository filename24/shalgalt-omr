import { invoke } from "@tauri-apps/api/core";

export interface ScanJob {
  task_id: string;
}

/** Rule 1 — `pdfPath` must always be an absolute path string. */
export function gradePdf(pdfPath: string, templateId: number): Promise<ScanJob> {
  return invoke("scan_grade_pdf", { pdfPath, templateId });
}
