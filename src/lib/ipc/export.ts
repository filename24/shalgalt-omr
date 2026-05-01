import { invoke } from "@tauri-apps/api/core";

/** P4: 결과 xlsx로 내보내기. 반환값은 저장된 파일의 절대경로. */
export function exportResultsXlsx(templateId: number): Promise<string> {
  return invoke("export_results_xlsx", { templateId });
}
