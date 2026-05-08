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
