/** 백엔드 `scan::pipeline::TaskStage`. */
export type TaskStage =
  | "loading_pdf"
  | "rasterizing"
  | "detecting_markers"
  | "reading_bubbles"
  | "grading"
  | "saving"
  | "done"
  | "failed";

/** Rule 2 — `task-progress` emit 페이로드. */
export interface TaskProgress {
  task_id: string;
  processed: number;
  total: number;
  stage: TaskStage;
  message?: string | null;
}
