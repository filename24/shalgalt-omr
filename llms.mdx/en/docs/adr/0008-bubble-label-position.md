# ADR 0008 — Bubble label position (https://filename24.github.io/shalgalt-omr/en/docs/adr/0008-bubble-label-position)



<Callout type="success" title="Accepted · 2026-05-09">
  **Deciders:** filename24 · **Related:** #75 (P2-06 follow-up), original ACK criterion in
  #17 (P2-06).
</Callout>

## Context [#context]

P2-06 acceptance criterion #17 specified &#x2A;"each circle carries a 1-character label drawn
**next to** it"*. The renderer faithfully implemented this and produced a card with
≈ 520 floating glyphs scattered around the bubble grid — 30 × 4 next to every Section-1
row plus ≈ 200 next to Шифр/Section-2 digit bubbles. The output was not usable as an
OMR card.

The acceptance criterion was wrong. The follow-up question — *where* should the label
character live? — has only a few credible answers and a real OMR-design tradition to
draw on.

## Options considered [#options-considered]

| Option                                   | Where the `'A'` glyph appears           | Real-world precedent                                     | CV impact                                              |
| ---------------------------------------- | --------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------ |
| A. Next to every bubble (P2-06 original) | tucked beside each circle               | Almost nobody                                            | Visual noise; CV unaffected                            |
| **B. Inside the circle**                 | centred inside the bubble outline       | **AMC** (`\AMCchoiceLabel`), most Mongolian-school cards | None — fully inked answer covers the printed glyph     |
| C. Column header strip per section       | one row of `A B C D E` above each block | OMRChecker, OpenMCR, Indian-style sheets                 | Adds a section-anchored marker; CV must skip the strip |
| D. No label, key only                    | nothing on the printed sheet            | Stylised, modern templates                               | Student must memorise meaning                          |

The market survey documented in `memory/project_omr_market_research.md`
shows AMC (FOSS reference implementation) lands on B. Mongolian general-school cards
match — see the reference photo at the time of issue #75. OMRChecker and OpenMCR
land on C, but their inputs are *bitmap* OMR sheets where the column header is also a
visual marker for the threshold pass; we generate the sheets ourselves and have ArUco
markers (P3-01) for that role.

## Decision [#decision]

<Callout type="info" title="Decision">
  Adopt **Option B — labels drawn INSIDE each circle.**
</Callout>

```rust
canvas.text_centered_in_circle(cx_mm, cy_mm, r_mm, ch, font);
```

implemented in `crates/shalgalt-pdf/src/canvas.rs` (see ADR 0007).
Auto-fits the font to ≈ 65 % of the bubble diameter (≈ 6 pt for the 3 mm bubble locked
in `docs/MONGOLIAN_OMR_SPEC.md` §4), within the OMR-spec
target range of 6–8 pt for in-bubble identifiers.

### Why this works for CV [#why-this-works-for-cv]

The CV pipeline (P3-01 / P3-02) thresholds the *fraction of dark pixels inside a
bubble*. The printed label is a thin glyph hull; a fully-inked answer fills the whole
circle and dwarfs the glyph's contribution. With:

* 3 mm bubble area = π × 1.5² = 7.07 mm²
* 6 pt glyph hull (rough): ≈ 1.0 mm²
* Threshold for "filled" (per master plan §6.5): bubble fraction ≥ 0.35

the printed label contributes ≈ 14 % fill, well below the 0.35 threshold. The
`crates/shalgalt-pdf/tests/labels_layout.rs` regression test
pins the upper bound on text-op count so the renderer cannot silently regress to
Option A and double-count labels.

### Section-header semantics [#section-header-semantics]

Sub-block stickers (`2.1`, `2.2`, `2.3`, `2.4`) and parent labels (`1-Р ХЭСЭГ`,
`2-Р ХЭСЭГ`) are emitted by `crates/shalgalt-pdf/src/layout/section_headers.rs`
and are NOT a column-header strip — they label the *block*, not each column position.
Шифр / Вариант suppress their headers (the geometry is self-evident).

## Consequences [#consequences]

### Positive [#positive]

* The card matches the visual idiom of every real Mongolian-school OMR sheet shipped
  to date.
* AMC interop (future LayoutMap sidecar — see master plan D4) becomes a coordinate
  translation problem only; the visual semantics already match.
* One glyph per bubble, period — no per-bubble label X/Y math at any call site.

### Negative [#negative]

* 3 mm bubbles + 6 pt in-bubble glyphs are at the small end of legibility. P5 may add a
  "comfortable" mode (4 mm bubble + 8 pt glyph) for low-vision printing, gated behind a
  user setting.
* The CV pipeline must not be tuned to look *for* the glyph (e.g., template-matching
  the printed `A`); it should treat the bubble as a uniform threshold target.

## Follow-up [#follow-up]

* The legibility / fill-fraction trade-off is parameterised in
  `crates/shalgalt-pdf/src/canvas.rs` — the
  `0.65` ratio in `text_centered_in_circle` is the lever to revise if classroom
  testing reports glyphs are bleeding into the answer mark.
* ADR 0007 covers the wrapper that this decision relies on.

<Cards>
  <Card href="/adr/0007-canvas-wrapper-and-layout-modules" title="ADR 0007 — Canvas wrapper & layout modules">
    The `Canvas` wrapper whose `text_centered_in_circle` helper this decision relies on.
  </Card>
  <Card href="/adr/0010-confidence-band" title="ADR 0010 — Fill measurement & confidence">
    How the CV pipeline measures bubble fill that this label placement must not disturb.
  </Card>
</Cards>
