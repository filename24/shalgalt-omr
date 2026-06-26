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
import { createExam } from "$lib/db/exams";
import { createAnswerKey, listAnswerKeysByExam } from "$lib/db/answerKeys";
import { toAnswerKey } from "$lib/types/exam";
import { parseStoredAnswerKeys } from "$lib/results/storedAnswerKeys";
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
  /**
   * When set, the exam's per-variant answer keys are bundled as
   * `answer-keys.json` (an `AnswerKey[]`) and the manifest records `exam_id`.
   */
  examId?: number;
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

  // 2. answer-keys.json — when an exam is bundled, ship every variant's key as
  // an `AnswerKey[]`. This is the canonical source for the import round-trip and
  // takes precedence over the job's single key (see the jobId branch below).
  let examKeysBundled = false;
  if (opts.examId !== undefined) {
    const records = await listAnswerKeysByExam(opts.examId);
    const keys = records.map(toAnswerKey);
    entries.push({
      kind: "inline",
      name: ENTRY_ANSWER_KEYS,
      content: JSON.stringify(keys),
    });
    examKeysBundled = true;
  }

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

    // answer-keys.json — the AnswerKey JSON the grade run was scored against.
    // Skipped when an exam already supplied the canonical keys above.
    if (!examKeysBundled) {
      entries.push({
        kind: "inline",
        name: ENTRY_ANSWER_KEYS,
        content: job.answer_key_json,
      });
    }

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
    // Record the source exam id when one was bundled. It is a provenance hint —
    // `restoreProject` always remaps keys onto a freshly created exam row.
    ...(opts.examId !== undefined ? { exam_id: opts.examId } : {}),
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
  const templateId = await createTemplate(template, null);

  // 2. Recreate the exam + its answer keys when the archive carries them.
  // `answer-keys.json` is either an `AnswerKey[]` (exam export) or a single
  // legacy `AnswerKey` (graded-job export); `parseStoredAnswerKeys` normalizes
  // both. Each key is validated, then remapped onto the freshly created exam
  // row — the source `exam_id` is provenance only and means nothing in this DB.
  const answerKeysText = await readEntry(opened, ENTRY_ANSWER_KEYS);
  if (answerKeysText) {
    const keys = parseStoredAnswerKeys(answerKeysText).map((k) =>
      answerKeySchema.parse(k),
    );
    const exam = await createExam({
      name: opened.manifest.title,
      template_id: templateId,
    });
    for (const key of keys) {
      await createAnswerKey({
        exam_id: exam.id,
        variant: key.variant,
        answers: key.answers,
      });
    }
  }

  // 3. Graded metadata + image rows are restored once the jobs/results import
  // path is wired. The extracted page images live under
  // `opened.workspaceDir`; their `ExtractedEntry.path` values are what future
  // result rows will point at. This is the largest remaining piece — it needs
  // a job row (createJob) plus per-page result reconstruction with remapped
  // image paths, which is substantial new import machinery beyond this task.
  // TODO(P4-05): rebuild job/result rows from metadata.json and link the
  // extracted page-*.png entries by path.
}
