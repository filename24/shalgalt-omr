/**
 * Mongolian standard OMR card preset.
 *
 * Implements the locked layout specification — see
 * [`docs/MONGOLIAN_OMR_SPEC.md`](../../../docs/MONGOLIAN_OMR_SPEC.md). Page is split
 * 25 % top (header + Шифр + Вариант + САНАМЖ instructions) / 75 % body (Section 1 +
 * Section 2). Coordinates are normalized [0,1] against an A4-portrait backdrop.
 *
 * Group inventory totals 107 groups (4 cipher rows + 1 variant row + 70 multi-choice
 * questions + 4×8 numeric digit rows). Section / label strings come from the i18n
 * table — this module owns coordinates only.
 */
import type {
  OmrTemplate,
  BubbleGroup,
  TemplatePoint
} from '$lib/types/template'
import { TEMPLATE_VERSION } from '$lib/types/template'
import { mn } from '$lib/i18n'

/** Section names — re-exported for tests and the LayerTree. */
export const STANDARD_SECTIONS = mn.editor.presets.sections

interface RowSpec {
  /** ID prefix for the generated group, e.g. "shifr-0". */
  idPrefix: string
  /** Human-readable label rendered in the inspector and result table. */
  label: string
  /** Section grouping shown in the LayerTree. */
  section: string
  /**
   * "student_id" for the cipher rows, "question" for graded rows, "variant" for
   * the exam-form selector (marked by the student but never graded).
   */
  kind: 'student_id' | 'question' | 'variant'
  /** Top-left bubble of the row in normalized coordinates. */
  origin: TemplatePoint
  /** Number of bubbles. */
  count: number
  /** Horizontal step between bubbles. */
  spacingX: number
  /** Optional pre-filled answer index (only meaningful for `kind === "question"`). */
  answerIndex?: number | null
  /** Score weight. Defaults to 1. */
  score?: number
}

function buildRow(spec: RowSpec): BubbleGroup {
  const bubbles: TemplatePoint[] = []
  for (let i = 0; i < spec.count; i++) {
    bubbles.push({
      x: spec.origin.x + spec.spacingX * i,
      y: spec.origin.y
    })
  }
  return {
    id: spec.idPrefix,
    kind: spec.kind,
    label: spec.label,
    bubbles,
    answer_index: spec.answerIndex ?? null,
    score: spec.score ?? 1,
    section: spec.section
  }
}

// ─── Layout constants ──────────────────────────────────────────────────────
//
// Coordinates are normalized [0,1] against the FULL A4-portrait page (matches
// the editor canvas, the CV pipeline's perspective warp, and the PDF
// renderer — see `crates/shalgalt-pdf/src/coords.rs`).
//
// Marker safe area: ArUco markers sit at template centres (0.04, 0.04) /
// (0.96, 0.04) / (0.96, 0.96) / (0.04, 0.96) with `size = 0.04`. Marker half-
// side projects to 4.2 mm on A4 (= 0.02 in normalized x, 0.014 in normalized y),
// so each marker square occupies:
//   x ∈ [0.02, 0.06] ∪ [0.94, 0.98]
//   y ∈ [0.026, 0.054] ∪ [0.946, 0.974]
// Bubble radius is 2.25 mm (= 0.011 in x, 0.008 in y). Every bubble centre
// below is therefore kept at least 0.025 in x / 0.018 in y away from the
// nearest marker edge so the printed circles never visually graze the
// ArUco grid.

const SHIFR = {
  section: STANDARD_SECTIONS.shifr,
  startY: 0.08,
  rowSpacing: 0.025,
  rows: 4,
  // Was 0.07 → bubble left edge at 0.0593 vs marker right edge at 0.06
  // (overlapping). Pushed right to clear the TL marker by ~5 mm.
  bubbleStartX: 0.085,
  bubbleSpacing: 0.032,
  bubbleCount: 10
} as const

const VARIANT = {
  section: STANDARD_SECTIONS.variant,
  y: 0.19,
  // Centred horizontally on the cipher block. Cipher span is now
  // 0.085 – (0.085 + 9 × 0.032) = 0.085 – 0.373, midpoint 0.229. With 4
  // bubbles × 0.032 spacing, the variant's first bubble sits at
  // 0.229 − 1.5 × 0.032 = 0.181.
  startX: 0.181,
  spacing: 0.032,
  count: 4
} as const

const SECTION_1 = {
  section: STANDARD_SECTIONS.section1,
  count: 70,
  perColumn: 35,
  // Was 0.02 → 35 rows packed to y = 0.95, overlapping the bottom markers
  // (top edge at y = 0.946). Tightened to 0.019 so the last row lands at
  // y = 0.916 with ~6.5 mm clearance.
  rowSpacing: 0.019,
  startY: 0.27,
  // Both columns shifted right to clear the TL/BL markers; gap between
  // columns preserved at 0.23 (was 0.07–0.30, now 0.085–0.315).
  leftColX: 0.085,
  rightColX: 0.315,
  bubbleSpacing: 0.032,
  bubbleCount: 5
} as const

const SECTION_2 = {
  rows: 8,
  // Was 0.020 — tightened to 0.019 to free room for the inter-block gap below
  // without pushing the last block past the bottom marker. Each block is now
  // 7 × 0.019 = 0.133 tall.
  rowSpacing: 0.019,
  // Was 0.18 → block 3 last row landed at y = 0.95, overlapping bottom
  // markers. A naïve drop to 0.16 erased the visible inter-block gap (the
  // "2.2"/"2.3"/"2.4" sub-headers ended up sitting on the previous block's
  // last row). Locked at 0.17 so the gap between blocks = 0.17 − 7 × 0.019
  // = 0.037 ≈ 11 mm — restores the visible separator while keeping block 3's
  // last row at y = 0.913 (~7.5 mm clear of the BR marker top edge).
  blockSpacingY: 0.17,
  startY: 0.27,
  startX: 0.55,
  bubbleCount: 10,
  bubbleSpacing: 0.032
} as const

const SECTION_2_BLOCKS = ['2.1', '2.2', '2.3', '2.4'] as const
const ROW_LABELS_2 = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'] as const
const VARIANT_LABELS = ['A', 'B', 'C', 'D', 'E'] as const

// ─── Section builders ──────────────────────────────────────────────────────

function buildShifrRows(): BubbleGroup[] {
  const out: BubbleGroup[] = []
  for (let r = 0; r < SHIFR.rows; r++) {
    out.push(
      buildRow({
        idPrefix: `shifr-${r}`,
        // Cipher rows render with no row label — the 4-row stack of 0–9 bubbles next to
        // the handwriting underline conveys "this is the cipher block" on its own.
        // Inspector / result table fall back to `idPrefix` when label is empty.
        label: '',
        section: SHIFR.section,
        kind: 'student_id',
        origin: {
          x: SHIFR.bubbleStartX,
          y: SHIFR.startY + SHIFR.rowSpacing * r
        },
        count: SHIFR.bubbleCount,
        spacingX: SHIFR.bubbleSpacing
      })
    )
  }
  return out
}

function buildVariantRow(): BubbleGroup {
  // Row label is suppressed — the section header above the bubble row already prints
  // "Хувилбар" (via section_headers.rs), so a row label would render the same word
  // twice on the same line.
  void VARIANT_LABELS
  return buildRow({
    idPrefix: 'variant',
    label: '',
    section: VARIANT.section,
    // The variant selector identifies which exam form the student sat. It is
    // marked like a question but must never be graded, so it carries its own
    // `variant` kind — excluded from the answer key and the scoring engine,
    // exactly like the `student_id` cipher rows.
    kind: 'variant',
    origin: { x: VARIANT.startX, y: VARIANT.y },
    count: VARIANT.count,
    spacingX: VARIANT.spacing,
    answerIndex: null
  })
}

function buildSection1Questions(): BubbleGroup[] {
  const out: BubbleGroup[] = []
  for (let q = 0; q < SECTION_1.count; q++) {
    const inLeftCol = q < SECTION_1.perColumn
    const indexInCol = inLeftCol ? q : q - SECTION_1.perColumn
    const x = inLeftCol ? SECTION_1.leftColX : SECTION_1.rightColX
    const y = SECTION_1.startY + SECTION_1.rowSpacing * indexInCol
    out.push(
      buildRow({
        idPrefix: `q-${q + 1}`,
        // Row label is just the question number ("1", "2", …, "70") — no "Q" prefix.
        // Saves horizontal space at 10 pt and matches the real Mongolian-school card.
        label: `${q + 1}`,
        section: SECTION_1.section,
        kind: 'question',
        origin: { x, y },
        count: SECTION_1.bubbleCount,
        spacingX: SECTION_1.bubbleSpacing,
        answerIndex: null
      })
    )
  }
  return out
}

function section2SectionLabel(id: (typeof SECTION_2_BLOCKS)[number]): string {
  switch (id) {
    case '2.1':
      return STANDARD_SECTIONS.section21
    case '2.2':
      return STANDARD_SECTIONS.section22
    case '2.3':
      return STANDARD_SECTIONS.section23
    case '2.4':
      return STANDARD_SECTIONS.section24
  }
}

function buildSection2Block(
  id: (typeof SECTION_2_BLOCKS)[number],
  blockIndex: number
): BubbleGroup[] {
  const out: BubbleGroup[] = []
  const startY = SECTION_2.startY + SECTION_2.blockSpacingY * blockIndex
  const section = section2SectionLabel(id)
  for (let r = 0; r < SECTION_2.rows; r++) {
    out.push(
      buildRow({
        idPrefix: `${id}-${ROW_LABELS_2[r]}`,
        label: `${id}.${ROW_LABELS_2[r]}`,
        section,
        kind: 'question',
        origin: { x: SECTION_2.startX, y: startY + SECTION_2.rowSpacing * r },
        count: SECTION_2.bubbleCount,
        spacingX: SECTION_2.bubbleSpacing,
        answerIndex: null
      })
    )
  }
  return out
}

/**
 * Build a fresh `OmrTemplate` matching the Mongolian general-school standard
 * layout. Used by the Toolbar's "New > Standard" entry. Returns 107 groups
 * (4 cipher + 1 variant + 70 questions + 4×8 numeric digit rows) laid out
 * across an A4-portrait coordinate space.
 */
export function createMongolianStandardTemplate(opts?: {
  title?: string
}): OmrTemplate {
  return {
    version: TEMPLATE_VERSION,
    title: opts?.title ?? '',
    // P3-01: ArUco DICT_6X6_50 markers. IDs are assigned in TL/TR/BR/BL order
    // so the CV pipeline can recover orientation even when the page is fed
    // upside-down or sideways through a scanner. See docs/adr/0009-aruco-markers.md.
    markers: [
      {
        id: 'm-tl',
        position: { x: 0.04, y: 0.04 },
        size: 0.04,
        kind: { type: 'aruco6x6', ids: [0, 1, 2, 3] }
      },
      {
        id: 'm-tr',
        position: { x: 0.96, y: 0.04 },
        size: 0.04,
        kind: { type: 'aruco6x6', ids: [0, 1, 2, 3] }
      },
      {
        id: 'm-br',
        position: { x: 0.96, y: 0.96 },
        size: 0.04,
        kind: { type: 'aruco6x6', ids: [0, 1, 2, 3] }
      },
      {
        id: 'm-bl',
        position: { x: 0.04, y: 0.96 },
        size: 0.04,
        kind: { type: 'aruco6x6', ids: [0, 1, 2, 3] }
      }
    ],
    groups: [
      ...buildShifrRows(),
      buildVariantRow(),
      ...buildSection1Questions(),
      ...SECTION_2_BLOCKS.flatMap((id, idx) => buildSection2Block(id, idx))
    ]
  }
}
