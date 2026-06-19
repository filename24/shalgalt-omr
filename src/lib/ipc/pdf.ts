import { invoke } from "@tauri-apps/api/core";
import type { OmrTemplate } from "$lib/types/template";

/**
 * Generate an OMR PDF from `template` and write it to `outputPath`.
 *
 * `outputPath` is the absolute path returned by the native save dialog
 * (`pickPdfSavePath`). Rule 1: only the path crosses the IPC boundary —
 * never the rendered bytes themselves.
 */
export function generateOmrPdf(args: {
  template: OmrTemplate;
  variant?: string;
  outputPath: string;
}): Promise<void> {
  return invoke("pdf_generate_omr", {
    templateJson: JSON.stringify(args.template),
    variant: args.variant ?? null,
    outputPath: args.outputPath,
  });
}

/**
 * Render `template` into a cached preview PNG and return its absolute path.
 *
 * The frontend feeds the path through `convertFileSrc` to load it via the
 * `asset://localhost/` protocol — never as a base64 data URL.
 */
export function renderPdfPreview(args: {
  template: OmrTemplate;
  variant?: string;
}): Promise<string> {
  return invoke("pdf_render_template_preview", {
    templateJson: JSON.stringify(args.template),
    variant: args.variant ?? null,
  });
}
