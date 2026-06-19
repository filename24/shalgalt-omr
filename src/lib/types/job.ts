/**
 * Frontend types for the `jobs` table (P3-08).
 *
 * The DB row shape (`Job`) is hand-written because there is no Rust counterpart
 * — `tauri-plugin-sql` owns the schema and ts-rs only mirrors `shalgalt-core`
 * domain types. The per-page payload (`GradedJobSheet`) reuses the generated
 * `ParsedSheet` / `GradedSheet` from the Rust core.
 */
import type { ParsedSheet } from "./generated/ParsedSheet";
import type { GradedSheet } from "./generated/GradedSheet";

export type JobStatus = "queued" | "running" | "done" | "failed" | "canceled";

/**
 * One graded page captured during a `scan_grade_pdf` run. Stored as an entry in
 * `jobs.graded_sheets_json`; `/review` mutates `graded` and `reviewed` while
 * the user disambiguates flagged bubbles.
 */
export interface GradedJobSheet {
  page_index: number;
  /** Absolute path to the rasterized page PNG. Load via `assetUrl(...)`. */
  page_image_path: string;
  parsed: ParsedSheet;
  graded: GradedSheet;
  /** True once the user explicitly marks the sheet reviewed in `/review`. */
  reviewed: boolean;
}

export interface Job {
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
