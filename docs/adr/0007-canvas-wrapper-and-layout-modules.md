# ADR 0007 — Canvas wrapper + layout-module split for `shalgalt-pdf`

- **Status**: Accepted
- **Date**: 2026-05-09
- **Deciders**: filename24
- **Related issues**: #75 (P2-06 follow-up)
- **Supersedes**: ADR 0002 §"Bubble geometry" (geometry now lives in
  [`docs/MONGOLIAN_OMR_SPEC.md`](../MONGOLIAN_OMR_SPEC.md))

## Context

P2-06 shipped a renderer where every layout module reached straight into `printpdf`'s
low-level `Op` enum. Within one P2-06 follow-up session four bug categories surfaced and
each had to be diagnosed in a *different* file:

1. **Cursor accumulation in BT/ET.** `Op::SetTextCursor` compiles to PDF `Td` (relative
   move). Two cursors inside one text section *add*. Bubble labels stacked on the first
   bubble of every group — diagnosed in `bubble_grid.rs`.
2. **Sidebar font fallback.** Mongolian Cyrillic instructions rendered as `.notdef`
   boxes when the caller passed `NotoSansMongolian` to a sidebar that should have used
   Latin/Cyrillic Noto Sans — diagnosed in `sidebar.rs` + `lib.rs`.
3. **Inside-circle digit position.** Centring math (offset by 30 % of glyph height)
   was duplicated wherever an in-bubble label was drawn — diagnosed in `bubble_grid.rs`.
4. **Row-label alignment.** Right-aligning a label against a bubble required hand-rolled
   width estimation at every call site — diagnosed in `labels.rs`.

Each of the four bugs was a one-line fix, but they all shared a symptom: low-level PDF
operators were being emitted from too many places, and the surrounding state (active
text section, current font, current text matrix) was implicit.

A market survey of mainstream OMR tools (AMC, Remark Office, OMRChecker, OpenMCR,
ZipGrade) confirmed that **none** drives `printpdf`-equivalent low-level ops directly:
AMC sits on `pdfTeX`, Remark on Word, the others ship pre-baked PDFs. The closest
prior art for our pure-Rust path is AMC's `automultiplechoice.sty` LaTeX wrapper, which
is *itself* a thin DSL over a lower-level engine.

Two coupled decisions land here so the next phase (P3 CV pipeline) does not have to
re-litigate them:

1. **Should layout modules emit raw `Op`s or go through a wrapper?**
2. **How should the layout submodules be organised?**

## Options considered

### Wrapper layer

| Option | Where the BT/ET / cursor / colour state lives | Cost to migrate |
| ------ | -------------------------------------------- | --------------- |
| A. Status quo (raw `Vec<Op>`) | scattered across `bubble_grid` / `labels` / `header` / `sidebar` / `manual_entry` | None |
| **B. `Canvas` thin wrapper** | one module owns BT/ET reset, font handle, colour, line width | ~150 LOC migration, +5 unit tests |
| C. `genpdf`-style layout engine | hidden — but layout decisions move into the engine | Rewrite, breaks Rule 3 contract |

### Module layout

| Option | Module count | Coupling |
| ------ | -----------: | -------- |
| **D. One submodule per visual element** (markers, header, bubble_grid, labels, manual_entry, numeric_block, section_headers, sidebar) | 8 | Each module imports `Canvas` + `coords` only |
| E. One mega-module `layout.rs` | 1 | Tight |
| F. Per-section modules (cipher, variant, section1, section2) | 4 | Each duplicates label / underline code |

## Decision

Adopt **B + D**.

### B — `Canvas` wrapper

`crates/shalgalt-pdf/src/canvas.rs` is the only module that emits text-related
`Op`s. Everything else calls intent-shaped helpers:

```rust
canvas.text(x_mm, y_mm, "Q1", font, 10.0);              // BT/ET wrapped per call
canvas.text_right_aligned(x_mm, y_mm, "70", font, 10.0);// width-estimated
canvas.text_centered_in_circle(cx, cy, r, '5', font);    // auto-fitted size
canvas.text_centered_x(x_mm, y_mm, "2.1", font, 8.0);   // sub-block sticker
canvas.circle_stroked(cx, cy, r);                        // delegates to shapes::
canvas.hline(x1, x2, y);                                 // handwriting underline
canvas.set_stroke_black(thickness_mm);                   // sticky state
```

Every `Canvas::text*` call wraps its own `StartTextSection` / `EndTextSection`. The
text matrix never accumulates across calls — bug #1 is mechanically impossible going
forward. Bug #3's centring math lives in one method. Bugs #2 and #4 reduce to "pass
the right `FontId`" / "use `text_right_aligned`" — both visible at the call site.

`Op` is still re-exported (`canvas::push`) as an escape hatch for primitives the
wrapper does not yet cover (graphics-state save/restore, text-matrix rotation in
`sidebar.rs`).

### D — Module-per-element layout

```
src/layout/
├── mod.rs
├── markers.rs           # 4 corner alignment squares
├── header.rs            # 2-column page header (title left + school right)
├── bubble_grid.rs       # circles only — labels owned by labels.rs
├── labels.rs            # row labels (right-aligned)
├── manual_entry.rs      # cipher handwriting underlines
├── numeric_block.rs     # delegates to bubble_grid
├── section_headers.rs   # 1-Р ХЭСЭГ / 2.x stickers
└── sidebar.rs           # top-right horizontal САНАМЖ block
```

Each module is ≤ 100 LOC. The split matches the spec sections in
[`docs/MONGOLIAN_OMR_SPEC.md`](../MONGOLIAN_OMR_SPEC.md) one-to-one — when the spec
gains a new visual element, exactly one new submodule lands.

## Consequences

### Positive

- The four cursor / font / centring / alignment bugs are now mechanically prevented
  rather than caught by code review.
- Layout modules read top-down as "draw circle, draw label, draw header" — the PDF
  vocabulary is gone.
- Adding a new visual element is one small file plus one register line in
  `layout/mod.rs`.
- Tests for layout primitives live in `canvas.rs` (5 unit tests pin BT/ET pairing,
  width estimation, in-circle centring).

### Negative

- One extra module + about 200 LOC of wrapper code.
- `printpdf` API changes are now centralised at the `Canvas` boundary — easier to
  update, but every renderer module reads against the wrapper, not the upstream.
- The Mongolian-script `sidebar.rs` still needs `Op` directly for the rotation-matrix
  case (kept as `canvas.push(Op::SetTextMatrix { … })`) — pure-wrapper migration is
  deferred until traditional Mongolian script support actually lands.

## Follow-up

- ADR 0008 documents the in-circle bubble-label decision that the Canvas wrapper
  enabled.
- Layout-spec changes follow the procedure in
  [`docs/MONGOLIAN_OMR_SPEC.md`](../MONGOLIAN_OMR_SPEC.md) §7 — issue + spec doc + TS
  preset + Rust fixture + golden regen, all in the same PR.
- A future LayoutMap sidecar (AMC's `.xy` pattern, master-plan D4) will plug into the
  Canvas at the call site without changing the wrapper API.
