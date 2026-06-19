/**
 * Repository for the `jobs` table (P3-08).
 *
 * Same pattern as `templates.ts` / `results.ts`: every call goes through
 * `getDb()`, queries are parameterized, and JSON-shaped columns are parsed
 * once at the boundary so callers see typed objects (`GradedJobSheet[]`),
 * never raw text.
 *
 * Lifecycle of a job row:
 *   1. `/grade` calls `createJob(...)` with `status: 'queued'` and the
 *      AnswerKey JSON pasted by the user.
 *   2. As `task-progress` events arrive, `updateJobProgress(...)` updates the
 *      counters; on the first event the row also moves to `status: 'running'`.
 *   3. On the terminal `done` / `failed` stage, `completeJob(...)` writes the
 *      collected `graded_sheets_json` and final `needs_review_count`.
 *   4. `/review/[job_id]` reads via `getJobById(...)`, mutates one sheet at a
 *      time through `updateGradedSheet(...)`.
 */
import { getDb } from "./index";
import type { GradedJobSheet, Job, JobStatus } from "$lib/types/job";

interface JobRow {
  id: number;
  task_id: string;
  pdf_path: string;
  template_id: number;
  answer_key_json: string;
  status: JobStatus;
  total_pages: number | null;
  processed_pages: number;
  needs_review_count: number;
  graded_sheets_json: string | null;
  error_message: string | null;
  created_at: string;
  updated_at: string;
}

const SELECT_COLUMNS = `id, task_id, pdf_path, template_id, answer_key_json,
  status, total_pages, processed_pages, needs_review_count,
  graded_sheets_json, error_message, created_at, updated_at`;

function rowToJob(r: JobRow): Job {
  return {
    id: r.id,
    task_id: r.task_id,
    pdf_path: r.pdf_path,
    template_id: r.template_id,
    answer_key_json: r.answer_key_json,
    status: r.status,
    total_pages: r.total_pages,
    processed_pages: r.processed_pages,
    needs_review_count: r.needs_review_count,
    graded_sheets_json: r.graded_sheets_json,
    error_message: r.error_message,
    created_at: r.created_at,
    updated_at: r.updated_at,
  };
}

export interface NewJobInput {
  task_id: string;
  pdf_path: string;
  template_id: number;
  answer_key_json: string;
}

export async function createJob(input: NewJobInput): Promise<Job> {
  const db = await getDb();
  const result = await db.execute(
    `INSERT INTO jobs (task_id, pdf_path, template_id, answer_key_json, status)
     VALUES ($1, $2, $3, $4, 'queued')`,
    [input.task_id, input.pdf_path, input.template_id, input.answer_key_json],
  );
  const id = Number(result.lastInsertId);
  const row = await db.select<JobRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM jobs WHERE id = $1`,
    [id],
  );
  if (row.length === 0) {
    throw new Error(`createJob: row ${id} disappeared after insert`);
  }
  return rowToJob(row[0]!);
}

export async function getJobByTaskId(taskId: string): Promise<Job | null> {
  const db = await getDb();
  const rows = await db.select<JobRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM jobs WHERE task_id = $1`,
    [taskId],
  );
  return rows.length > 0 ? rowToJob(rows[0]!) : null;
}

export async function getJobById(id: number): Promise<Job | null> {
  const db = await getDb();
  const rows = await db.select<JobRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM jobs WHERE id = $1`,
    [id],
  );
  return rows.length > 0 ? rowToJob(rows[0]!) : null;
}

export async function listRecentJobs(limit = 25): Promise<Job[]> {
  const db = await getDb();
  const rows = await db.select<JobRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM jobs ORDER BY created_at DESC, id DESC LIMIT $1`,
    [limit],
  );
  return rows.map(rowToJob);
}

/**
 * One completed grading job as surfaced in the results browser (P5-02). Joins
 * the template title and keeps the lightweight counters — no `graded_sheets_json`
 * blob, so the list view never carries page payloads across the wire.
 */
export interface ResultJobRow {
  id: number;
  template_id: number;
  template_title: string;
  total_pages: number | null;
  processed_pages: number;
  needs_review_count: number;
  created_at: string;
}

export interface ResultJobFilter {
  /** Restrict to one template, or `null`/omitted for all templates. */
  templateId?: number | null;
  /** Only jobs with at least one sheet still flagged for review. */
  needsReviewOnly?: boolean;
  limit: number;
  offset: number;
}

/** Shared WHERE clause + bound params for the results-browser queries. */
function resultJobWhere(filter: ResultJobFilter): {
  clause: string;
  params: (string | number)[];
} {
  const conditions = ["j.status = 'done'"];
  const params: (string | number)[] = [];
  if (filter.templateId != null) {
    params.push(filter.templateId);
    conditions.push(`j.template_id = $${params.length}`);
  }
  if (filter.needsReviewOnly) {
    conditions.push("j.needs_review_count > 0");
  }
  return { clause: conditions.join(" AND "), params };
}

/** Page of completed jobs for the results browser, newest first. */
export async function listResultJobs(
  filter: ResultJobFilter,
): Promise<ResultJobRow[]> {
  const db = await getDb();
  const { clause, params } = resultJobWhere(filter);
  const limitIdx = params.length + 1;
  const offsetIdx = params.length + 2;
  return db.select<ResultJobRow[]>(
    `SELECT j.id, j.template_id, COALESCE(t.title, '') AS template_title,
            j.total_pages, j.processed_pages, j.needs_review_count, j.created_at
     FROM jobs j
     LEFT JOIN templates t ON t.id = j.template_id
     WHERE ${clause}
     ORDER BY j.created_at DESC, j.id DESC
     LIMIT $${limitIdx} OFFSET $${offsetIdx}`,
    [...params, filter.limit, filter.offset],
  );
}

/** Total count for pagination, honoring the same filter. */
export async function countResultJobs(
  filter: ResultJobFilter,
): Promise<number> {
  const db = await getDb();
  const { clause, params } = resultJobWhere(filter);
  const rows = await db.select<{ n: number }[]>(
    `SELECT COUNT(*) AS n FROM jobs j WHERE ${clause}`,
    params,
  );
  return rows[0]?.n ?? 0;
}

export interface JobProgressUpdate {
  processed_pages: number;
  total_pages: number | null;
}

export async function updateJobProgress(
  taskId: string,
  p: JobProgressUpdate,
): Promise<void> {
  const db = await getDb();
  await db.execute(
    `UPDATE jobs
     SET status = CASE WHEN status = 'queued' THEN 'running' ELSE status END,
         processed_pages = $1,
         total_pages = COALESCE($2, total_pages)
     WHERE task_id = $3`,
    [p.processed_pages, p.total_pages, taskId],
  );
}

export interface JobCompletion {
  status: "done" | "failed" | "canceled";
  graded_sheets: GradedJobSheet[] | null;
  needs_review_count: number;
  error_message?: string | null;
}

export async function completeJob(
  taskId: string,
  c: JobCompletion,
): Promise<void> {
  const db = await getDb();
  const json = c.graded_sheets ? JSON.stringify(c.graded_sheets) : null;
  await db.execute(
    `UPDATE jobs
     SET status = $1,
         graded_sheets_json = $2,
         needs_review_count = $3,
         error_message = $4
     WHERE task_id = $5`,
    [c.status, json, c.needs_review_count, c.error_message ?? null, taskId],
  );
}

/**
 * Decode `graded_sheets_json` from a job row. Returns an empty array when the
 * column is null (job not yet completed).
 */
export function parseGradedSheets(job: Job): GradedJobSheet[] {
  if (!job.graded_sheets_json) return [];
  return JSON.parse(job.graded_sheets_json) as GradedJobSheet[];
}

/**
 * Replace one sheet inside a job's `graded_sheets_json` blob and recompute
 * `needs_review_count` from the new array. Used by `/review` when the user
 * overrides bubble readings or marks a sheet reviewed.
 */
export async function updateGradedSheet(
  jobId: number,
  pageIndex: number,
  sheet: GradedJobSheet,
): Promise<void> {
  const db = await getDb();
  const rows = await db.select<JobRow[]>(
    `SELECT ${SELECT_COLUMNS} FROM jobs WHERE id = $1`,
    [jobId],
  );
  if (rows.length === 0) {
    throw new Error(`updateGradedSheet: job ${jobId} not found`);
  }
  const job = rowToJob(rows[0]!);
  const sheets = parseGradedSheets(job);
  const next = sheets.map((s) => (s.page_index === pageIndex ? sheet : s));
  const needsReview = next.filter(
    (s) => s.graded.needs_review && !s.reviewed,
  ).length;
  await db.execute(
    `UPDATE jobs
     SET graded_sheets_json = $1,
         needs_review_count = $2
     WHERE id = $3`,
    [JSON.stringify(next), needsReview, jobId],
  );
}
