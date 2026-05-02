import type { OmrTemplate, TemplateSummary } from "$lib/types/template";

/**
 * Currently-edited template (P1+).
 *
 * P0 keeps this as a plain rune-based holder. The editor route (P1) will
 * mutate `draft` while bubble groups / markers are dragged on the konva
 * canvas; persistence happens through `lib/db/templates.ts` after explicit
 * Save actions. `pristine` keeps the last-saved snapshot for diff/undo.
 */
class CurrentTemplateStore {
  draft = $state<OmrTemplate | null>(null);
  pristine = $state<OmrTemplate | null>(null);
  loadedFrom = $state<TemplateSummary | null>(null);

  load(summary: TemplateSummary): void {
    this.loadedFrom = summary;
    this.draft = structuredClone(summary.schema);
    this.pristine = structuredClone(summary.schema);
  }

  reset(): void {
    this.draft = null;
    this.pristine = null;
    this.loadedFrom = null;
  }

  get isDirty(): boolean {
    if (!this.draft || !this.pristine) return false;
    return JSON.stringify(this.draft) !== JSON.stringify(this.pristine);
  }
}

export const currentTemplate = new CurrentTemplateStore();
