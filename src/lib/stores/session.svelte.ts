/**
 * App-wide session state (P0 stub).
 *
 * Holds short-lived UI affordances that outlive a single route — e.g. the
 * active grade job's `task_id`, the last-picked PDF path, the most recently
 * opened template id. Persistence belongs in SQLite (`lib/db/`); this store
 * is in-memory only and resets on app restart.
 */
class SessionStore {
  activeJobId = $state<string | null>(null);
  lastPdfPath = $state<string | null>(null);
  lastOpenedTemplateId = $state<number | null>(null);

  setActiveJob(taskId: string | null): void {
    this.activeJobId = taskId;
  }

  recordPdfPath(path: string): void {
    this.lastPdfPath = path;
  }

  recordTemplate(id: number): void {
    this.lastOpenedTemplateId = id;
  }
}

export const session = new SessionStore();
