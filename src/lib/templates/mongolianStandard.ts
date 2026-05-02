/**
 * Mongolian standard OMR card preset.
 *
 * Inspired by the de-facto layout used in Mongolian general-education schools
 * (4-digit student cipher, 1 variant selector, 30 multiple-choice questions in
 * two columns, then two numeric sub-sections of 8 single-digit rows each).
 * The visual design is our own — the school-specific PDF that motivated this
 * preset is NOT bundled.
 *
 * Coordinates are normalized [0,1] against an A4-portrait backdrop. They are
 * rough-but-plausible defaults; teachers fine-tune in the editor against their
 * own scanned blank form.
 *
 * Group inventory totals 51 groups (4 cipher rows + 1 variant row + 30
 * multiple-choice questions + 8 + 8 numeric digit rows). Section / label
 * strings come from the i18n table — this module owns coordinates only.
 */
import type { OmrTemplate, BubbleGroup, TemplatePoint } from "$lib/types/template";
import { TEMPLATE_VERSION } from "$lib/types/template";
import { mn } from "$lib/i18n";

/** Section names — re-exported for tests and the LayerTree. */
export const STANDARD_SECTIONS = mn.editor.presets.sections;

interface RowSpec {
  /** ID prefix for the generated group, e.g. "shifr-0". */
  idPrefix: string;
  /** Human-readable label rendered in the inspector and result table. */
  label: string;
  /** Section grouping shown in the LayerTree. */
  section: string;
  /** "student_id" for input rows, "question" for graded rows. */
  kind: "student_id" | "question";
  /** Top-left bubble of the row in normalized coordinates. */
  origin: TemplatePoint;
  /** Number of bubbles. */
  count: number;
  /** Horizontal step between bubbles. */
  spacingX: number;
  /** Optional pre-filled answer index (only meaningful for `kind === "question"`). */
  answerIndex?: number | null;
  /** Score weight. Defaults to 1. */
  score?: number;
}

function buildRow(spec: RowSpec): BubbleGroup {
  const bubbles: TemplatePoint[] = [];
  for (let i = 0; i < spec.count; i++) {
    bubbles.push({
      x: spec.origin.x + spec.spacingX * i,
      y: spec.origin.y,
    });
  }
  return {
    id: spec.idPrefix,
    kind: spec.kind,
    label: spec.label,
    bubbles,
    answer_index: spec.answerIndex ?? null,
    score: spec.score ?? 1,
    section: spec.section,
  };
}

// ─── Layout constants ──────────────────────────────────────────────────────
//
// The PDF inspiration uses A4 portrait; coordinates below are normalized [0,1]
// against that page rect. Adjusting any constant updates every group derived
// from it.

const SHIFR = {
  section: STANDARD_SECTIONS.shifr,
  startY: 0.155,
  rowSpacing: 0.024,
  rows: 4,
  bubbleStartX: 0.075,
  bubbleSpacing: 0.0255,
  bubbleCount: 10,
} as const;

const VARIANT = {
  section: STANDARD_SECTIONS.variant,
  y: 0.27,
  startX: 0.10,
  spacing: 0.026,
  count: 4,
} as const;

const SECTION_1 = {
  section: STANDARD_SECTIONS.section1,
  count: 30,
  perColumn: 15,
  rowSpacing: 0.02,
  startY: 0.36,
  leftColX: 0.36,
  rightColX: 0.61,
  bubbleSpacing: 0.03,
  bubbleCount: 4,
} as const;

const SECTION_2 = {
  rows: 8,
  rowSpacing: 0.018,
  startY: 0.74,
  bubbleCount: 10,
  bubbleSpacing: 0.026,
  leftColX: 0.07,
  rightColX: 0.55,
} as const;

const ROW_LABELS_2 = ["a", "b", "c", "d", "e", "f", "g", "h"] as const;
const VARIANT_LABELS = ["A", "B", "C", "D"] as const;

// ─── Section builders ──────────────────────────────────────────────────────

function buildShifrRows(): BubbleGroup[] {
  const out: BubbleGroup[] = [];
  for (let r = 0; r < SHIFR.rows; r++) {
    out.push(
      buildRow({
        idPrefix: `shifr-${r}`,
        label: `${mn.editor.presets.labels.shifrRow}-${r + 1}`,
        section: SHIFR.section,
        kind: "student_id",
        origin: { x: SHIFR.bubbleStartX, y: SHIFR.startY + SHIFR.rowSpacing * r },
        count: SHIFR.bubbleCount,
        spacingX: SHIFR.bubbleSpacing,
      }),
    );
  }
  return out;
}

function buildVariantRow(): BubbleGroup {
  return buildRow({
    idPrefix: "variant",
    label: `${mn.editor.presets.labels.variant} (${VARIANT_LABELS.join("/")})`,
    section: VARIANT.section,
    kind: "question",
    origin: { x: VARIANT.startX, y: VARIANT.y },
    count: VARIANT.count,
    spacingX: VARIANT.spacing,
    answerIndex: null,
  });
}

function buildSection1Questions(): BubbleGroup[] {
  const out: BubbleGroup[] = [];
  for (let q = 0; q < SECTION_1.count; q++) {
    const inLeftCol = q < SECTION_1.perColumn;
    const indexInCol = inLeftCol ? q : q - SECTION_1.perColumn;
    const x = inLeftCol ? SECTION_1.leftColX : SECTION_1.rightColX;
    const y = SECTION_1.startY + SECTION_1.rowSpacing * indexInCol;
    out.push(
      buildRow({
        idPrefix: `q-${q + 1}`,
        label: `${mn.editor.presets.labels.question}${q + 1}`,
        section: SECTION_1.section,
        kind: "question",
        origin: { x, y },
        count: SECTION_1.bubbleCount,
        spacingX: SECTION_1.bubbleSpacing,
        answerIndex: null,
      }),
    );
  }
  return out;
}

function buildSection2Block(id: "2.1" | "2.2"): BubbleGroup[] {
  const out: BubbleGroup[] = [];
  const startX = id === "2.1" ? SECTION_2.leftColX : SECTION_2.rightColX;
  const section = id === "2.1" ? STANDARD_SECTIONS.section21 : STANDARD_SECTIONS.section22;
  for (let r = 0; r < SECTION_2.rows; r++) {
    out.push(
      buildRow({
        idPrefix: `${id}-${ROW_LABELS_2[r]}`,
        label: `${id}.${ROW_LABELS_2[r]}`,
        section,
        kind: "question",
        origin: { x: startX, y: SECTION_2.startY + SECTION_2.rowSpacing * r },
        count: SECTION_2.bubbleCount,
        spacingX: SECTION_2.bubbleSpacing,
        answerIndex: null,
      }),
    );
  }
  return out;
}

/**
 * Build a fresh `OmrTemplate` matching the Mongolian general-school standard
 * layout. Used by the Toolbar's "New > Standard" entry. Returns 51 groups
 * laid out across an A4-portrait coordinate space.
 */
export function createMongolianStandardTemplate(opts?: { title?: string }): OmrTemplate {
  return {
    version: TEMPLATE_VERSION,
    title: opts?.title ?? "",
    markers: [
      { id: "m-tl", position: { x: 0.04, y: 0.04 }, size: 0.02 },
      { id: "m-tr", position: { x: 0.96, y: 0.04 }, size: 0.02 },
      { id: "m-br", position: { x: 0.96, y: 0.96 }, size: 0.02 },
      { id: "m-bl", position: { x: 0.04, y: 0.96 }, size: 0.02 },
    ],
    groups: [
      ...buildShifrRows(),
      buildVariantRow(),
      ...buildSection1Questions(),
      ...buildSection2Block("2.1"),
      ...buildSection2Block("2.2"),
    ],
  };
}
