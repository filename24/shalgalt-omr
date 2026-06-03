import { invoke } from "@tauri-apps/api/core";

/**
 * Plaintext `manifest.json` header mirrored from
 * `shalgalt_fileformat::Manifest`. Fields stay snake_case to match the Rust
 * serde output (the manifest is serialized as-is through the command layer).
 * `exam_id` / `hint` are omitted from the JSON when absent on the Rust side.
 */
export interface ProjectManifest {
  format_version: number;
  title: string;
  created_at: string;
  sheet_count: number;
  encrypted: boolean;
  exam_id?: number;
  hint?: string;
}

/**
 * A single entry the frontend hands to `project_save` / `project_export`.
 *
 * Rule 1 — never send bytes across IPC. The split mirrors the Rust
 * `ProjectEntry` enum:
 *  - `inline`: small text payloads (template.json, answer-keys.json,
 *    metadata.json, students.csv) are passed as a UTF-8 `content` string and
 *    written into the zip from memory.
 *  - `file`: large binaries (scanned PDFs, page images) are passed by absolute
 *    `sourcePath`; Rust streams them from disk so their bytes never cross the
 *    IPC boundary.
 *
 * The Rust side tags the enum on `kind` (camelCase), so the JS object shape is
 * exactly `{ kind, name, content }` or `{ kind, name, sourcePath }`.
 */
export type ProjectEntry =
  | { kind: "inline"; name: string; content: string }
  | { kind: "file"; name: string; sourcePath: string };

/** One file unpacked from a `.shalgalt` container into the workspace dir. */
export interface ExtractedEntry {
  /** Logical name inside the archive (e.g. `template.json`). */
  name: string;
  /** Absolute path to the extracted file in the workspace directory. */
  path: string;
}

/** Result of `project_open`: the plaintext manifest plus extracted entries. */
export interface OpenedProject {
  manifest: ProjectManifest;
  /** Absolute path to the temp/workspace directory holding the extracted files. */
  workspaceDir: string;
  entries: ExtractedEntry[];
}

export interface ProjectWriteArgs {
  /** Serialized `Manifest` JSON; always stored plaintext in the container. */
  manifestJson: string;
  /** Canonical order: template.json, answer-keys.json, metadata.json, students.csv, then *.pdf / images. */
  entries: ProjectEntry[];
  /** Absolute destination path (Rule 1 — picked via the save dialog). */
  outputPath: string;
  /** Optional age passphrase; when present the payload is encrypted. */
  passphrase?: string;
}

/**
 * Persist the current project to a `.shalgalt` container at `outputPath`.
 *
 * Used by the in-app "Save" flow where the destination is the project's own
 * file. Encryption is applied when `passphrase` is supplied.
 */
export function projectSave(args: ProjectWriteArgs): Promise<void> {
  return invoke("project_save", {
    manifestJson: args.manifestJson,
    entries: args.entries,
    outputPath: args.outputPath,
    passphrase: args.passphrase,
  });
}

/**
 * Export the current project to a `.shalgalt` container at `outputPath`.
 *
 * Same wire shape as `projectSave`; kept distinct so the host can apply
 * export-specific semantics (e.g. "Save a Copy") without overloading save.
 */
export function projectExport(args: ProjectWriteArgs): Promise<void> {
  return invoke("project_export", {
    manifestJson: args.manifestJson,
    entries: args.entries,
    outputPath: args.outputPath,
    passphrase: args.passphrase,
  });
}

export interface ProjectOpenArgs {
  /** Absolute path to the `.shalgalt` file (Rule 1 — picked via the open dialog). */
  inputPath: string;
  /** Required only when the container was encrypted; otherwise omit. */
  passphrase?: string;
}

/**
 * Open a `.shalgalt` container: read its plaintext manifest, extract every
 * entry into a workspace directory on disk, and return the manifest plus the
 * extracted file paths (Rule 1 — the frontend loads them by path, never bytes).
 */
export function projectOpen(args: ProjectOpenArgs): Promise<OpenedProject> {
  return invoke("project_open", {
    inputPath: args.inputPath,
    passphrase: args.passphrase,
  });
}
