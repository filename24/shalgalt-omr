/** 백엔드 `domain::template::TemplatePoint`. */
export interface TemplatePoint {
  x: number;
  y: number;
}

/** 백엔드 `domain::template::Marker`. */
export interface Marker {
  id: string;
  position: TemplatePoint;
  size: number;
}

/** 백엔드 `domain::template::BubbleKind`. */
export type BubbleKind = "student_id" | "question";

/** 백엔드 `domain::template::BubbleGroup`. */
export interface BubbleGroup {
  id: string;
  kind: BubbleKind;
  label: string;
  bubbles: TemplatePoint[];
  answer_index: number | null;
  score: number;
}

/** Rule 3 — `templates.json_schema`로 직렬화되는 최상위 구조. */
export interface OmrTemplate {
  version: number;
  title: string;
  markers: [Marker, Marker, Marker, Marker];
  groups: BubbleGroup[];
}

/** `commands::templates::TemplateSummary`. */
export interface TemplateSummary {
  id: number;
  title: string;
  schema: OmrTemplate;
}
