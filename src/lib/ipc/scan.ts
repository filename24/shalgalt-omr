import { invoke } from "@tauri-apps/api/core";

export interface ScanJob {
  task_id: string;
}

/** Rule 1: pdfPath는 항상 절대 경로 문자열이어야 한다. */
export function gradePdf(pdfPath: string, templateId: number): Promise<ScanJob> {
  return invoke("scan_grade_pdf", { pdfPath, templateId });
}
