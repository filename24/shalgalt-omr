import { z } from "zod";

/** Mirrors Rust `domain::template::TemplatePoint`. */
export interface TemplatePoint {
  x: number;
  y: number;
}

/** Mirrors Rust `domain::template::Marker`. */
export interface Marker {
  id: string;
  position: TemplatePoint;
  size: number;
}

/** Mirrors Rust `domain::template::BubbleKind`. */
export type BubbleKind = "student_id" | "question";

/** Mirrors Rust `domain::template::BubbleGroup`. */
export interface BubbleGroup {
  id: string;
  kind: BubbleKind;
  label: string;
  bubbles: TemplatePoint[];
  answer_index: number | null;
  score: number;
  /**
   * Optional UI-only grouping label (e.g. cipher / variant / section names).
   * Pure organization metadata for the LayerTree and result tables — the CV
   * pipeline and grading engine ignore it.
   */
  section?: string;
}

/** Rule 3 — top-level structure persisted into `templates.json_schema`. */
export interface OmrTemplate {
  version: number;
  title: string;
  markers: [Marker, Marker, Marker, Marker];
  groups: BubbleGroup[];
}

/** Mirrors `commands::templates::TemplateSummary`. */
export interface TemplateSummary {
  id: number;
  title: string;
  schema: OmrTemplate;
  /** Absolute path to a sidecar backdrop image stored in `$APPDATA`. Sidecar — not part of `OmrTemplate`. */
  backdropPath?: string;
}

/** Current schema version. Bumps trigger a forward-compat migration in P5. */
export const TEMPLATE_VERSION = 1;

const templatePointSchema = z.object({
  x: z.number(),
  y: z.number(),
});

const markerSchema = z.object({
  id: z.string().min(1),
  position: templatePointSchema,
  size: z.number().nonnegative(),
});

const bubbleKindSchema = z.union([z.literal("student_id"), z.literal("question")]);

const bubbleGroupSchema = z.object({
  id: z.string().min(1),
  kind: bubbleKindSchema,
  label: z.string(),
  bubbles: z.array(templatePointSchema),
  answer_index: z.number().int().nonnegative().nullable(),
  score: z.number().nonnegative(),
  section: z.string().optional(),
});

/**
 * Runtime guard for the JSON stored in `templates.json_schema`.
 *
 * Used at every DB-load boundary (see `src/lib/db/templates.ts`) to make a
 * silently-corrupt template surface as a typed error instead of leaking
 * garbage into the editor state.
 */
export const templateSchema = z.object({
  version: z.number().int().positive(),
  title: z.string(),
  markers: z.tuple([markerSchema, markerSchema, markerSchema, markerSchema]),
  groups: z.array(bubbleGroupSchema),
});

export type TemplateSchemaT = z.infer<typeof templateSchema>;

/**
 * Build a fresh `OmrTemplate` with four corner markers near the page edges and
 * no bubble groups. The backdrop reference lives outside the template per
 * Rule 3, so this factory does not take a backdrop argument.
 */
export function createEmptyTemplate(opts?: { title?: string }): OmrTemplate {
  return {
    version: TEMPLATE_VERSION,
    // Default title is supplied by the caller (typically `mn.editor.untitled`)
    // so the type module stays decoupled from i18n.
    title: opts?.title ?? "",
    markers: [
      { id: "m-tl", position: { x: 0.05, y: 0.05 }, size: 0.02 },
      { id: "m-tr", position: { x: 0.95, y: 0.05 }, size: 0.02 },
      { id: "m-br", position: { x: 0.95, y: 0.95 }, size: 0.02 },
      { id: "m-bl", position: { x: 0.05, y: 0.95 }, size: 0.02 },
    ],
    groups: [],
  };
}
