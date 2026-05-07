import type { OmrTemplate, TemplateSummary } from "$lib/types/template";

/**
 * Currently-edited template.
 *
 * `draft` is mutated in-place by the editor (marker drag, bubble group
 * changes, inspector field edits). `pristine` is a frozen snapshot of the
 * last-loaded or last-saved state; comparing against it gives the dirty flag.
 *
 * Backdrop path is tracked alongside the template but is *not* part of
 * `OmrTemplate` itself — Rule 3 keeps the persisted JSON resolution-agnostic
 * and machine-portable. The path lives in the sidecar `backdrop_path` column
 * (see migration 0002) and is loaded/saved separately.
 *
 * `draftUuid` is a frontend-only id used to bucket on-disk template assets
 * under `$APPDATA/templates/<uuid>/` while the template is still in-flight
 * (i.e. before it gets a SQLite row id). It survives across saves — the UUID
 * folder is never renamed; the row's `backdrop_path` simply points into it.
 */
class CurrentTemplateStore {
  draft = $state<OmrTemplate | null>(null);
  pristine = $state<OmrTemplate | null>(null);
  loadedFrom = $state<TemplateSummary | null>(null);
  backdropPath = $state<string | null>(null);
  pristineBackdropPath = $state<string | null>(null);
  draftUuid = $state<string | null>(null);

  load(summary: TemplateSummary): void {
    this.loadedFrom = summary;
    this.draft = structuredClone(summary.schema);
    this.pristine = structuredClone(summary.schema);
    this.backdropPath = summary.backdropPath ?? null;
    this.pristineBackdropPath = summary.backdropPath ?? null;
    // A loaded row keeps its existing on-disk asset folder; we do not try to
    // recover the original UUID, so the draft uuid stays null. New asset
    // imports while editing a loaded template will prompt for a fresh uuid.
    this.draftUuid = null;
  }

  /**
   * Initialize a brand-new in-flight draft. Caller passes the freshly-minted
   * uuid (typically `crypto.randomUUID()`) so on-disk asset paths can be
   * derived consistently across components.
   */
  initDraft(template: OmrTemplate, uuid: string): void {
    this.loadedFrom = null;
    this.draft = template;
    this.pristine = structuredClone(template);
    this.backdropPath = null;
    this.pristineBackdropPath = null;
    this.draftUuid = uuid;
  }

  reset(): void {
    this.draft = null;
    this.pristine = null;
    this.loadedFrom = null;
    this.backdropPath = null;
    this.pristineBackdropPath = null;
    this.draftUuid = null;
  }

  get isDirty(): boolean {
    if (!this.draft || !this.pristine) return false;
    if (this.backdropPath !== this.pristineBackdropPath) return true;
    return JSON.stringify(this.draft) !== JSON.stringify(this.pristine);
  }
}

export const currentTemplate = new CurrentTemplateStore();
