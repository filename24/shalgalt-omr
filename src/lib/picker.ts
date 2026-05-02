import { open } from "@tauri-apps/plugin-dialog";

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
