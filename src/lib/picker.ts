import { open, save } from "@tauri-apps/plugin-dialog";

/**
 * Open a native file picker for a single PDF and return its absolute path.
 *
 * Returns `null` when the user cancels. The returned string is what Rule 1
 * mandates we send across IPC — never the file bytes themselves.
 */
export async function pickPdf(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  return typeof selected === "string" ? selected : null;
}

/**
 * Dialog filters for a grading scan source. The first entry lists every accepted
 * extension so a teacher can pick a PDF or a phone photo without switching the
 * dialog's type dropdown; the rest narrow to PDF-only or image-only.
 */
const SCAN_SOURCE_FILTERS = [
  {
    name: "Шалгалтын хуудас",
    extensions: ["pdf", "png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff"],
  },
  { name: "PDF", extensions: ["pdf"] },
  { name: "Зураг", extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff"] },
];

/**
 * Open a native file picker for a single grading scan source — either a
 * multi-page PDF or one scanned image (PNG/JPG/JPEG/WebP/BMP/TIFF) — and return
 * its absolute path. Returns `null` when the user cancels. Same Rule 1 invariant
 * as `pickPdf`: we hand the path across IPC, never the bytes.
 */
export async function pickScanSource(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: SCAN_SOURCE_FILTERS,
  });
  return typeof selected === "string" ? selected : null;
}

/**
 * Open a native file picker for one or more grading scan sources — any mix of
 * multi-page PDFs and single scanned images — and return their absolute paths in
 * the order the OS reports them. Returns an empty array when the user cancels.
 *
 * This is what powers multi-page upload: each image counts as one page, so a
 * teacher can select a whole stack of phone photos (and/or PDFs) in one go and
 * grade them as a single batch. Same Rule 1 invariant as `pickPdf`.
 */
export async function pickScanSources(): Promise<string[]> {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: SCAN_SOURCE_FILTERS,
  });
  if (Array.isArray(selected)) return selected;
  return typeof selected === "string" ? [selected] : [];
}

/**
 * Open a native file picker for a single backdrop image (PNG/JPG/JPEG/WebP)
 * and return its absolute path. Same Rule 1 invariant as `pickPdf`.
 */
export async function pickImage(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg", "webp"] }],
  });
  return typeof selected === "string" ? selected : null;
}

/**
 * Open the native "Save As" dialog seeded with `defaultName` and return the
 * absolute output path picked by the user, or `null` when they cancel.
 * The save-dialog scope grants Tauri write permission for the chosen path —
 * the path stays inside the OS picker boundary even when the destination is
 * outside the default `$APPDATA` capability scope.
 */
export async function pickPdfSavePath(defaultName: string): Promise<string | null> {
  const selected = await save({
    defaultPath: defaultName,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  return typeof selected === "string" ? selected : null;
}

/**
 * Open the native "Save As" dialog seeded with `defaultName` and return the
 * absolute `.xlsx` output path, or `null` when the user cancels. Same
 * save-dialog scope semantics as `pickPdfSavePath` — the chosen path is handed
 * to `export_results_xlsx`, which writes the workbook Rust-side (Rule 1).
 */
export async function pickXlsxSavePath(defaultName: string): Promise<string | null> {
  const selected = await save({
    defaultPath: defaultName,
    filters: [{ name: "Excel", extensions: ["xlsx"] }],
  });
  return typeof selected === "string" ? selected : null;
}

/**
 * Open a native file picker for a single `.shalgalt` project file and return
 * its absolute path. Same Rule 1 invariant as `pickPdf` — we hand the path to
 * `project_open`, never the bytes.
 */
export async function pickShalgalt(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Shalgalt project", extensions: ["shalgalt"] }],
  });
  return typeof selected === "string" ? selected : null;
}

/**
 * Open the native "Save As" dialog seeded with `defaultName` and return the
 * absolute `.shalgalt` output path, or `null` when the user cancels. Same
 * save-dialog scope semantics as `pickPdfSavePath`.
 */
export async function pickShalgaltSavePath(defaultName: string): Promise<string | null> {
  const selected = await save({
    defaultPath: defaultName,
    filters: [{ name: "Shalgalt project", extensions: ["shalgalt"] }],
  });
  return typeof selected === "string" ? selected : null;
}
