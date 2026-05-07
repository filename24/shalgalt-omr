import { invoke } from "@tauri-apps/api/core";

export interface ScanJob {
  task_id: string;
}

/** Rule 1 — `pdfPath` must always be an absolute path string. */
export function gradePdf(pdfPath: string, templateId: number): Promise<ScanJob> {
  return invoke("scan_grade_pdf", { pdfPath, templateId });
}

/**
 * Rasterize the first page of a PDF into a PNG via pdfium-render and return
 * the absolute path to the cached image. Used by the P1 template editor to
 * import a PDF as the canvas backdrop.
 */
export function rasterizePdfFirstPage(pdfPath: string): Promise<string> {
  return invoke("rasterize_pdf_first_page", { pdfPath });
}
