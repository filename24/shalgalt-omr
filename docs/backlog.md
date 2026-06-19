# shalgalt-omr — backlog

Non-blocking ideas surfaced during planning or implementation that do not need to land
in v1.0. Each entry should be self-contained — name, motivation, sketch, and a clear
"won't do" trigger so we know when to drop it.

> Locked v1.0 decisions live in
> [`.claude/PRPs/plans/shalgalt-omr-master.plan.md`](../.claude/PRPs/plans/shalgalt-omr-master.plan.md)
> §6, NOT here. ADRs in [`docs/adr/`](adr/) supersede this document for any item that
> moves into the locked set.

---

## D6 — Field-block sugar in `OmrTemplate`

**Surfaced**: P2-06 follow-up (PR #76) — OMR market research dated 2026-05-09.

OMRChecker (`Udayraj123/OMRChecker`) describes Section-2-style numeric blocks as a
single field block with a count and a row direction:

```jsonc
{
  "block": "2.1",
  "rows": ["a", "b", "c", "d", "e", "f", "g", "h"],
  "digits": 10,
  "origin": [0.55, 0.27],
  "row_pitch": 0.020,
  "bubble_pitch": 0.032
}
```

Our current `OmrTemplate` (Rule 3) flattens that into 8 separate `BubbleGroup`
entries per sub-block — 32 groups for one Section 2 = 32 × ≈ 80 bytes of
coordinate JSON. A field-block sugar layer would compress repetitive numeric
blocks ≈ 10× without changing the on-disk schema (Rule 3) — the desugar happens
on load.

**Why it is not in v1.0**

- The `BubbleGroup` flattening is correct and the editor already drives it
  programmatically via the preset builders (`buildSection2Block` in TS,
  `section2_block` in Rust).
- The compression would mainly help hand-edited templates, which is not a v1.0
  user story (the editor is the supported authoring path).
- Adding it later is a non-breaking change — desugaring on load preserves the
  serialised `BubbleGroup` shape.

**Trigger to revisit**

- Template files exceed ~50 kB on disk and become slow to load.
- Users start hand-editing `template.json` and want a more compact format.
- A second numeric-block-style preset lands and the duplication in TS / Rust
  preset builders becomes painful to keep in sync.

**Sketch**

- Add a `field_blocks: Vec<FieldBlock>` slot to `OmrTemplate` (additive,
  defaulted to empty so existing templates round-trip).
- On load, expand each `FieldBlock` into the equivalent `BubbleGroup`s before
  passing to the renderer / grader. The desugar is pure and unit-testable.
- The editor stays group-centric; only the load/save layer knows about the sugar.

**Cross-references**

- Master plan §6.2 / Rule 3 (`OmrTemplate` JSON contract).
- ADR 0008 (in-circle bubble label) — unchanged.
- Memory: `project_omr_market_research.md` records the OMRChecker comparison.

---

## D5 — ArUco identity encoding (capture exam_id + variant in marker bits)

**Surfaced**: P2-06 follow-up (PR #76) — OMR market research dated 2026-05-09.

AMC's binary-box variant strip encodes `(exam_id, copy, page)` into 12 + 6 + 6
bits of printed dots. ArUco `DICT_6X6_50` carries 50 distinct IDs per dictionary;
choosing different IDs for the four corner markers can absorb the same identity
bits without a separate strip — the strip becomes redundant.

**Why it is not in v1.0**

- P3-01 currently treats all four ArUco markers as identical fiducials for
  perspective-warp only. Identity decoding is additional CV work + grading-side
  bookkeeping.
- The variant-bubble row already covers single-paper variant disambiguation
  (`A` / `B` / `C` / `D`); cross-paper exam_id reconciliation is a P5+ concern
  when `.shalgalt` containers start crossing institutional boundaries.

**Trigger to revisit**

- A school requests "scan a stack of mixed-variant papers without manual
  sorting" — that is the user story this would unlock.
- We add server-side grading (P5) and want the scanned PDF to self-identify
  against the `.shalgalt` exam_id without the user picking one in the UI.

**Sketch**

- Pick 4 ArUco IDs per `(exam_id, page)` pair from a dictionary slice.
- The grader reads marker IDs alongside positions and matches them against the
  loaded `.shalgalt` manifest to confirm correct exam.
- A mismatched-marker scan surfaces an `exam_mismatch` error with a Mongolian
  message in the i18n table.

**Cross-references**

- Master plan §6.4 (ArUco markers).
- Issue #24 — left a tracking comment when the v1.0 ArUco work closes.

---

## Comfortable-mode bubble size (P5 accessibility)

**Surfaced**: ADR 0008 §Negative.

The locked 3 mm bubble + ≈ 6 pt in-bubble glyph is at the legibility floor for
older / low-vision printing. A user-toggle "comfortable" mode (4 mm bubble +
8 pt glyph, ≈ 7.5 mm pitch) would re-flow Sections 1 + 2 to fewer rows per page
but improve readability.

**Why it is not in v1.0**

- The locked spec is calibrated for the 70-question Mongolian-school card. A
  comfortable mode changes question density and forces a layout-spec branch.
- P5 already owns the accessibility audit — the toggle naturally lands there.

**Trigger to revisit**

- Classroom testing reports glyphs are bleeding into answer marks at 3 mm.
- A user with reduced vision reports the locked card is unusable.

**Sketch**

- New `BubbleStyle::comfortable()` + `OmrTemplate.layout_mode: "standard" | "comfortable"`.
- Comfortable mode lays out 50 questions instead of 70 (split into two pages
  for full-class exams).

**Cross-references**

- Master plan §7 (Design system for older teachers).
- ADR 0008 §Negative.
