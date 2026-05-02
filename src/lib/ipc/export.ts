import { invoke } from "@tauri-apps/api/core";

/** P4: export results to xlsx. Returns the absolute path of the saved file. */
export function exportResultsXlsx(templateId: number): Promise<string> {
  return invoke("export_results_xlsx", { templateId });
}
