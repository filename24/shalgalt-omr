# `crates/shalgalt-pdf` — OMR PDF Renderer

Pure-Rust OMR sheet generator backed by [`printpdf`]. Embeds Noto Sans (Latin + Cyrillic)
and Noto Sans Mongolian fonts. Reads `OmrTemplate` from `shalgalt-core`. No `tauri`,
OpenCV, or pdfium dependency — reusable from `apps/server` and future CLI tools.

> Repo-level rules, language conventions, and locked decisions live in
> [`/AGENTS.md`](../../AGENTS.md). This file describes only what is specific to this crate.

## Module Layout

```
src/
  lib.rs               — `render_template(template, opts) -> Vec<u8>` entry point.
  coords.rs            — Normalized [0,1] template coords ↔ printpdf bottom-left mm.
                         `to_page_mm` (full page, markers) + `project` (margin-aware, body).
  shapes.rs            — Vector primitives (circle, rect) used by every layout module.
  style.rs             — `BubbleStyle`, `HeaderText` knobs.
  error.rs             — `PdfError` (thiserror-typed crate boundary).
  layout/
    markers.rs         — Four corner alignment markers (Rule 3 anchors).
    header.rs          — Title / school / teacher banner.
    bubble_grid.rs     — Per-`BubbleGroup` labelled circles.
    numeric_block.rs   — Vertical Шифр / Section-2 number columns.
    sidebar.rs         — Right-edge 90°-rotated Mongolian-script instructions.
assets/fonts/          — `NotoSans-Regular.ttf`, `NotoSansMongolian-Regular.ttf`
                         embedded via `include_bytes!`. Subsetted at save time.
tests/
  smoke.rs             — Renders a fixture template and asserts %PDF- header.
  preset_smoke.rs      — Mongolian-standard preset round-trip.
  golden.rs            — Bytewise comparison against `tests/golden_data/*.pdf`
                         (with `/ID` stripped — see Determinism below).
```

## Public Surface

```rust
pub fn render_template(template: &OmrTemplate, opts: &PdfOptions) -> Result<Vec<u8>, PdfError>;
```

`PdfOptions` knobs: paper size, variant tag (`"A"`/`"B"`), answer-key overlay flag (P5),
header text (`HeaderText`), Mongolian-script sidebar instructions, choice labels (default
`A..E`), digit labels (default `0..9`), bubble visual style, optional `created_at`
override for deterministic timestamps.

The returned `Vec<u8>` is a complete PDF byte stream from the `%PDF-` header through
`%%EOF`, ready to write to disk or stream over HTTP.

## Critical Rules — pdf renderer

### Rule 3 — Coordinate System

Templates use normalized coordinates in `[0, 1]` (master plan §9.2). printpdf's native
coordinate system is bottom-left origin in millimetres. The translation is centralized
in `coords.rs`:

- `coords::to_page_mm(point, paper)` — full-page projection. Used for the four corner
  markers because they sit outside the body margin.
- `coords::project(point, paper)` — margin-aware projection. Used for everything else.

Never re-derive coordinate math at the call site. If a layout module needs a different
projection, add a named function to `coords.rs` and document the use case there.

### Determinism

`shalgalt-pdf` outputs are byte-stable across runs and across operating systems. Three
properties guarantee this:

1. `PdfSaveOptions::subset_fonts` is enabled. Only glyphs that were actually drawn end up
   in the file — preview generation does not blow up the cache with unused codepoints.
2. `PdfDocument::new` defaults `creation_date` / `modification_date` to the unix epoch.
   Callers can override via `PdfOptions::created_at`.
3. printpdf randomizes the `/ID` array per run. The golden-file test in `tests/golden.rs`
   strips `/ID` before comparing bytes. New tests that compare bytes MUST do the same.

If you change `lib.rs` and golden tests start failing, regenerate the goldens with
`cargo test -p shalgalt-pdf --test golden -- --nocapture` after manually inspecting the
diff. Never silently overwrite the goldens.

## Fonts

Two fonts are embedded:

- `NotoSans-Regular.ttf` — Latin + Cyrillic body. Always loaded.
- `NotoSansMongolian-Regular.ttf` — Traditional Mongolian script. Loaded only when the
  caller passes `instructions: Some(_)`. Skipping the font when the sidebar is empty
  keeps preview cache files small.

Both fonts ship under their original Open Font License — see `assets/fonts/LICENSE`.

## Testing

```bash
cargo test -p shalgalt-pdf                # smoke + preset + golden
cargo test -p shalgalt-pdf --test golden  # only golden comparison
```

Golden data lives in `tests/golden_data/`. The `tests/common/` module contains shared
helpers for stripping `/ID` and `/DocChecksum` from PDF bytes before comparison.

## What does NOT belong here

- pdfium-render — it is a runtime PDF parser, not a writer. The CV pipeline rasterizes
  PDFs back to images and lives in `shalgalt-cv` instead.
- Translation logic — `instructions` and `header` text arrive pre-translated. The i18n
  table lives in the SvelteKit frontend.
- Tauri / OpenCV deps — would pollute `apps/server`'s dependency graph.
