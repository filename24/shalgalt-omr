---
title: "ADR 0002 — PDF generator: printpdf"
description: Adopt the pure-Rust printpdf engine with embedded Noto fonts and lock the OMR bubble geometry the CV pipeline depends on.
---

<Callout type="success" title="Accepted · 2026-05-08">
  **Deciders:** filename24 · **Related issues:** #16 (P2-05), #17 (P2-06)
</Callout>

## Context

P2-05 brought the `shalgalt-pdf` crate online, P2-06 turns it into a real OMR card
renderer (markers, header, multiple-choice grid, numeric blocks, Mongolian-script
sidebar). Two coupled decisions land here so P3 (CV pipeline) does not re-litigate them:

1. **PDF engine choice** — what library writes the PDF bytes, and what determinism /
   font / dependency budget we commit to.
2. **OMR bubble geometry** — physical diameter, stroke width, and pitch of every printed
   bubble. The CV pipeline (P3-01 / P3-02) hard-codes these into its blob detector and
   adaptive threshold parameters, so a renderer that chooses a different value would
   silently degrade grading accuracy.

## Options considered (engine)

| Option | Pure Rust | Embed TTF | Determinism | C deps | Lic | Note |
| ------ | :-------: | :-------: | :---------: | :----: | :-: | ---- |
| **A. `printpdf` 0.9** | **Yes** | **Yes** | **Yes (epoch default)** | **None** | **MIT/Apache** | Chosen |
| B. `genpdf` | Yes | Via `rusttype` | Yes | None | MPL-2.0 | Higher-level layout, but layout engine fights us for vector primitives |
| C. `lopdf` (raw) | Yes | Manual | Yes | None | MIT | Too low-level — every text run becomes a hand-written content stream |
| D. `pdfium-render` (write) | No | Via pdfium | No | libpdfium | Apache | Ships a 30+ MB native lib; overkill for write-only |
| E. `weasyprint` / HTML→PDF | No | Yes | Probabilistic CSS layout | Python + Cairo + Pango | LGPL | Hostile to a Tauri sidecar and to determinism |

## Decision

### Engine

<Callout type="info" title="Decision">
  Adopt **Option A — `printpdf` 0.9.x**.
</Callout>

Rationale (priority order):

1. **Pure Rust, no system deps.** `shalgalt-pdf` builds on every target the workspace
   already supports without adding the OpenCV-style Windows install dance. This was the
   single biggest factor.
2. **Deterministic output.** `PdfDocument::new` defaults the `creation_date` /
   `modification_date` to the unix epoch. The trailer's `/ID` array is randomized per
   run, but is straightforward to strip in golden-file comparisons (see
   `tests/golden.rs::normalize_pdf`). This makes the renderer suitable for the
   golden-file regression workflow that lands in P2-12.
3. **TTF embedding + auto-subsetting.** `PdfSaveOptions { subset_fonts: true, .. }`
   keeps the output reasonable when only a small Cyrillic / Mongolian-script glyph set
   is actually drawn.
4. **No higher-level layout engine.** The renderer is data-driven from `OmrTemplate`,
   which already encodes every coordinate. A library that re-imposes its own flow
   layout (genpdf, weasyprint) just gets in the way.

### Bubble geometry (locked for P3-01 / P3-02)

| Property | Value |
| -------- | ----- |
| Bubble diameter | **2.5 mm** |
| Stroke width | **0.4 mm** |
| Center-to-center pitch (multiple choice) | **6.0 mm** |
| Center-to-center pitch (numeric digit row) | **5.4 mm** |
| Label offset (right of bubble) | **1.5 mm** |
| Label font size | **8 pt** Noto Sans Regular |
| Marker side length | **`size * min(width, height)` mm** (template-driven; default `size = 0.02` → ~4.2 mm on A4) |

These match the defaults in `BubbleStyle::default()` (`crates/shalgalt-pdf/src/style.rs`).
The pitch values are emergent from the Mongolian-standard preset
(`src/lib/templates/mongolianStandard.ts`); changing the preset coordinates in either
language without updating the other will be caught by the golden-file test.

### Determinism contract

- The renderer never reads wall-clock time on its own.
- Callers may override `PdfOptions::created_at`. The default is `None`, in which case
  `printpdf`'s epoch default is used.
- Every other source of randomness (`/ID`) is stripped in golden comparisons. Production
  builds keep the random `/ID` for PDF-spec compliance.

## Consequences

- **CV team contract.** P3-01 / P3-02 may assume the values above when designing blob
  detector parameters and template-fit tolerances. Any future change to the geometry
  requires a new ADR and a corresponding update to the golden file (regenerated via
  `cargo test -p shalgalt-pdf --test golden -- --ignored regenerate_golden`).
- **printpdf 0.9 → 0.10 migration.** The `Op` enum and `ParsedFont::from_bytes`
  signatures changed between 0.7 → 0.8 → 0.9. Pin minor versions; revisit with an ADR
  follow-up before bumping past 0.9.x.
- **Mongolian script.** `NotoSansMongolian-Regular.ttf` is embedded only when the
  caller passes `instructions: Some(_)`. Vertical script rendering uses
  `Op::SetTextMatrix(TextMatrix::TranslateRotate(x, y, 90.0))` — `printpdf`'s `Tm`
  operator path. Real shaping (joining contextual forms) is out of scope; if the
  Mongolian sidebar reads poorly in production we revisit with `harfbuzz_rs`.
- **Future ArUco markers.** `MarkerKind::Aruco6x6` is currently rendered as a solid
  square placeholder. Bitmap stamping ships with the CV migration in P3-01.

## Follow-up

- P2-12 — record this ADR alongside the rest of the P2 ADR set.
- P3-01 — document the CV-side acceptance numbers (blob radius window, threshold
  parameters) that were calibrated against the geometry above.

<Cards>
  <Card href="/adr/0007-canvas-wrapper-and-layout-modules" title="ADR 0007 — Canvas wrapper & layout modules">
    The drawing-surface abstraction and per-element layout modules built on top of this engine.
  </Card>
  <Card href="/adr/0009-aruco-markers" title="ADR 0009 — ArUco corner markers">
    Replaces the placeholder square markers with `DICT_6X6_50` bitmaps during the P3-01 CV migration.
  </Card>
</Cards>
