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
}
