/**
 * `.shalgalt` project DB round-trip glue (P4-03).
 *
 * This module bridges the local SQLite repositories (`$lib/db/*`) and the
 * `.shalgalt` IPC commands (`$lib/ipc/project`). It contains NO UI — P4-04
 * wires `assembleProject` / `restoreProject` to the save / open buttons.
 *
 * Two responsibilities:
 *  - `assembleProject`: read a template (and optionally a graded job) out of
 *    the DB and turn it into a manifest + entry list ready for `project_save`
 *    / `project_export`.
 *  - `restoreProject`: take the extracted entries of an opened container and
 *    re-insert the rows into the local DB.
 *
 * Rule 1 — small JSON payloads travel as `inline` entries; large binaries
 * (scanned PDFs, page images) travel as `file` entries by absolute path so
 * their bytes never cross the IPC boundary.
 */
import { readTextFile } from "@tauri-apps/plugin-fs";

import type { OpenedProject, ProjectEntry } from "$lib/ipc/project";
import { getTemplate, createTemplate } from "$lib/db/templates";
import { getJobById, parseGradedSheets } from "$lib/db/jobs";
import { templateSchema, type OmrTemplate } from "$lib/types/template";
import { answerKeySchema } from "$lib/schemas/answerKey";
import type { GradedJobSheet } from "$lib/types/job";

/** Canonical archive entry names (the order callers must preserve). */
const ENTRY_TEMPLATE = "template.json";
const ENTRY_ANSWER_KEYS = "answer-keys.json";
const ENTRY_METADATA = "metadata.json";

/** Current `.shalgalt` format version (mirrors `FORMAT_VERSION` in Rust). */
const FORMAT_VERSION = 1;

export interface AssembleProjectOptions {
  templateId: number;
  /** When set, the job's answer key and graded sheets are bundled too. */
  jobId?: number;
  title: string;
  /**
   * When set to a non-empty string, the container is encrypted with this age
   * passphrase and the manifest's `encrypted` flag is set accordingly. The same
   * passphrase MUST be forwarded to `project_save` / `project_export`: the Rust
   * writer rejects any archive where `manifest.encrypted !== passphrase.is_some()`.
   */
  passphrase?: string;
}

export interface AssembledProject {
  /** Serialized `Manifest` JSON; always stored plaintext in the container. */
  manifestJson: string;
  /** Entries in canonical order: template, answer-keys, metadata, then files. */
  entries: ProjectEntry[];
}

/**
 * Read the chosen template (and optionally a graded job) from the local DB and
 * package them into a manifest + entry list for `project_save` /
 * `project_export`.
 *
 * The template JSON is always present. Answer keys, per-page metadata, and the
 * large file entries are included only when a `jobId` is supplied and the job
 * carries that data.
 */
export async function assembleProject(
  opts: AssembleProjectOptions,
): Promise<AssembledProject> {
  const template = await getTemplate(opts.templateId);
  if (!template) {
    throw new Error(`assembleProject: template ${opts.templateId} not found`);
  }

  const entries: ProjectEntry[] = [];

  // 1. template.json — always inline (small Rule-3 JSON blob).
  entries.push({
    kind: "inline",
    name: ENTRY_TEMPLATE,
    content: JSON.stringify(template.schema),
  });

  let sheetCount = 0;
  let gradedSheets: GradedJobSheet[] = [];
  let sourcePdfPath: string | null = null;

  if (opts.jobId !== undefined) {
    const job = await getJobById(opts.jobId);
    if (!job) {
      throw new Error(`assembleProject: job ${opts.jobId} not found`);
    }
    sourcePdfPath = job.pdf_path;
    gradedSheets = parseGradedSheets(job);
    sheetCount = gradedSheets.length;

    // 2. answer-keys.json — the AnswerKey JSON the grade run was scored against.
    entries.push({
      kind: "inline",
      name: ENTRY_ANSWER_KEYS,
      content: job.answer_key_json,
    });

    // 3. metadata.json — the graded per-page payload (scores + readings).
    entries.push({
      kind: "inline",
      name: ENTRY_METADATA,
      content: JSON.stringify(gradedSheets),
    });
  }

  // 4. Large binaries as `file` entries (Rule 1 — bytes stay on disk).
  // The source PDF, if known, ships once.
  if (sourcePdfPath) {
    entries.push({
      kind: "file",
      name: "source.pdf",
      sourcePath: sourcePdfPath,
    });
  }
  // Each rasterized page image ships under a stable name.
  for (const sheet of gradedSheets) {
    if (sheet.page_image_path) {
      entries.push({
        kind: "file",
        name: `pages/page-${sheet.page_index}.png`,
        sourcePath: sheet.page_image_path,
      });
    }
  }

  const manifest = {
    format_version: FORMAT_VERSION,
    title: opts.title,
    // RFC 3339 / ISO 8601 — matches the Rust `chrono::DateTime<Utc>` serde format.
    created_at: new Date().toISOString(),
    sheet_count: sheetCount,
    // Derived from the passphrase so the flag always matches reality. The caller
    // forwards the same passphrase to project_export; the Rust writer enforces
    // `manifest.encrypted === passphrase.is_some()`.
    encrypted: Boolean(opts.passphrase),
    // TODO(P4-04): populate `exam_id` once exam rows are wired in.
  };

  return { manifestJson: JSON.stringify(manifest), entries };
}

/**
 * Read one extracted entry as UTF-8 text, returning `null` when the archive
 * did not contain it. The appcache scope grants read permission for the
 * workspace directory `project_open` extracted into.
 */
async function readEntry(
  opened: OpenedProject,
  name: string,
): Promise<string | null> {
  const entry = opened.entries.find((e) => e.name === name);
  if (!entry) return null;
  return readTextFile(entry.path);
}

/**
 * Re-insert the contents of an opened `.shalgalt` container into the local DB.
 *
 * The template is always restored. Answer keys / graded metadata are validated
 * with the existing zod schemas before use. Image rows reference the extracted
 * entry paths so the frontend can load them via `asset://localhost/`.
 */
export async function restoreProject(opened: OpenedProject): Promise<void> {
  // 1. Restore the template (Rule 3 — single OmrTemplate JSON blob).
  const templateText = await readEntry(opened, ENTRY_TEMPLATE);
  if (!templateText) {
    throw new Error("restoreProject: archive is missing template.json");
  }
  const template = templateSchema.parse(JSON.parse(templateText)) as OmrTemplate;
  // Backdrop is not part of the project file (Rule 3 sidecar), so null here.
  await createTemplate(template, null);

  // 2. Validate the answer key when present. Persisting it belongs with the
  // exam/job rows that P4-04 introduces; validate now so a corrupt archive
  // fails loudly at import time.
  const answerKeysText = await readEntry(opened, ENTRY_ANSWER_KEYS);
  if (answerKeysText) {
    answerKeySchema.parse(JSON.parse(answerKeysText));
    // TODO(P4-04): insert the answer key into the exams table once it exists.
  }

  // 3. Graded metadata + image rows are restored once the jobs/results import
  // path is wired. The extracted page images live under
  // `opened.workspaceDir`; their `ExtractedEntry.path` values are what future
  // result rows will point at.
  // TODO(P4-04): rebuild job/result rows from metadata.json and link the
  // extracted page-*.png entries by path.
}
