/** Mirrors Rust `scan::pipeline::TaskStage`. */
export type TaskStage =
  | "loading_pdf"
  | "rasterizing"
  | "detecting_markers"
  | "reading_bubbles"
  | "grading"
  | "saving"
  | "done"
  | "failed";

/** Rule 2 — payload of the `task-progress` event. */
export interface TaskProgress {
  task_id: string;
  processed: number;
  total: number;
  stage: TaskStage;
  message?: string | null;
}
