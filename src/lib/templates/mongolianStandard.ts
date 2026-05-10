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
  /** "student_id" for input rows, "question" for graded rows. */
  kind: 'student_id' | 'question'
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
// The PDF inspiration uses A4 portrait; coordinates below are normalized [0,1]
// against that page rect. Adjusting any constant updates every group derived
// from it.

const SHIFR = {
  section: STANDARD_SECTIONS.shifr,
  startY: 0.08,
  rowSpacing: 0.025,
  rows: 4,
  bubbleStartX: 0.07,
  bubbleSpacing: 0.032,
  bubbleCount: 10
} as const

const VARIANT = {
  section: STANDARD_SECTIONS.variant,
  y: 0.19,
  // Centred horizontally on the cipher block (cipher span 0.07 – 0.358, midpoint
  // 0.214). With 4 bubbles × 0.032 spacing, the variant's first bubble must sit at
  // 0.214 − 1.5 × 0.032 = 0.166 so its 4 bubbles straddle the cipher midpoint.
  startX: 0.166,
  spacing: 0.032,
  count: 4
} as const

const SECTION_1 = {
  section: STANDARD_SECTIONS.section1,
  count: 70,
  perColumn: 35,
  rowSpacing: 0.02,
  startY: 0.27,
  leftColX: 0.07,
  rightColX: 0.3,
  bubbleSpacing: 0.032,
  bubbleCount: 5
} as const

const SECTION_2 = {
  rows: 8,
  rowSpacing: 0.02,
  blockSpacingY: 0.18,
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
    kind: 'question',
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
 * layout. Used by the Toolbar's "New > Standard" entry. Returns 51 groups
 * laid out across an A4-portrait coordinate space.
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
