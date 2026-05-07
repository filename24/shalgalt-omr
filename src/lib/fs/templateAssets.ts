/**
 * Template asset filesystem helpers.
 *
 * Each in-flight template gets a UUID-named folder under
 * `<appData>/templates/<uuid>/` to bucket its sidecar assets (currently just
 * the backdrop image). On first save the folder name is kept as-is — the row
 * holds the absolute path on `templates.backdrop_path`, so renaming buys us
 * nothing and risks broken references.
 *
 * Rule 1 — file bytes never travel through IPC. Image imports use
 * `plugin-fs.copyFile` (an OS-level copy) and PDF imports go through the
 * `rasterize_pdf_first_page` Rust command which writes the PNG itself.
 */
import { appDataDir, join } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";
import { copyFile, mkdir, remove } from "@tauri-apps/plugin-fs";
import { rasterizePdfFirstPage } from "$lib/ipc/scan";

async function templateDir(templateUuid: string): Promise<string> {
  return join(await appDataDir(), "templates", templateUuid);
}

function extensionOf(absPath: string): string {
  const dot = absPath.lastIndexOf(".");
  if (dot < 0 || dot === absPath.length - 1) return "png";
  return absPath.slice(dot + 1).toLowerCase();
}

/**
 * Copy a user-picked image into the per-template asset folder and return the
 * absolute path of the copy. The destination respects the `assetProtocol.scope`
 * already configured for `$APPDATA/**`, so `assetUrl(...)` will resolve.
 */
export async function importImageBackdrop(
  absSource: string,
  templateUuid: string,
): Promise<string> {
  const dir = await templateDir(templateUuid);
  await mkdir(dir, { recursive: true });
  const dest = await join(dir, `backdrop.${extensionOf(absSource)}`);
  await copyFile(absSource, dest);
  return dest;
}

/**
 * Render the first page of a PDF via Rust+pdfium, then copy the cached PNG
 * into the per-template asset folder. Returns the absolute path to the copy
 * inside `$APPDATA/templates/<uuid>/`.
 */
export async function importPdfBackdrop(
  absPdfSource: string,
  templateUuid: string,
): Promise<string> {
  const cachePng = await rasterizePdfFirstPage(absPdfSource);
  const dir = await templateDir(templateUuid);
  await mkdir(dir, { recursive: true });
  const dest = await join(dir, "backdrop.png");
  await copyFile(cachePng, dest);
  return dest;
}

/** Wrapper around Tauri's `convertFileSrc` so callers do not need to import the api. */
export function assetUrl(absPath: string): string {
  return convertFileSrc(absPath);
}

/**
 * Best-effort cleanup of a per-template asset folder. Used when the user
 * discards an unsaved draft — the row was never created, so the folder would
 * otherwise leak.
 */
export async function purgeTemplateAssets(templateUuid: string): Promise<void> {
  try {
    const dir = await templateDir(templateUuid);
    await remove(dir, { recursive: true });
  } catch {
    // Folder may not exist yet (no backdrop was ever imported). Silent ignore.
  }
}
