# `crates/shalgalt-cv` — OMR Computer-Vision Pipeline

PDF rasterization (pdfium-render) + OMR algorithms (OpenCV). The crate that turns scanned
PDFs into `GradedSheet` rows. Re-exports `TaskProgress` / `TaskStage` from `shalgalt-core`
for IPC consumers.

> Repo-level rules, language conventions, and locked decisions live in
> [`/AGENTS.md`](../../AGENTS.md). This file describes only what is specific to this crate.

## Module Layout

```
src/
  lib.rs           — `process_sources(source_paths, template, progress_tx)` grading
                     facade + `read_answer_key(...)`. Sources are a batch of PDFs and/or
                     single images, flattened into one continuous page sequence.
  pdf.rs           — pdfium-render multi-page rasterization + single-image transcode.
                     `rasterize_sources` (batch) / `rasterize_source` (one) / `is_image_path`.
  preview.rs       — Single-page rasterization used by the P1 template editor.
  perspective.rs   — ArUco corner detection (≥3 of 4) + `findHomography` warp (P3).
  bubbles.rs       — Per-bubble fill-ratio reading + confidence scoring (P3).
tests/
  pdfium_marker_raster.rs — Regression test that feeds bytes from `shalgalt-pdf` into
                            pdfium-render and asserts the four corner markers contain
                            dark pixels. Dev-only — runtime stays free of pdfium↔pdf
                            crate cycles.
```

## System Dependencies

- **OpenCV** is a build-time prerequisite (`libopencv-dev` on Linux, `brew install opencv`
  on macOS, the prebuilt self-extractor on Windows). See repo README and ADR 0001.
- **pdfium binaries** — pdfium-render dynamically loads `libpdfium`. The release pipeline
  (P6) ships the binary alongside the desktop app; for `cargo test` locally, see the
  pdfium-render docs.
- **clang** — opencv-rust uses libclang for code generation.

## Critical Rules — cv pipeline

### Rule 1 — Path arguments only

Every public function MUST take filesystem paths (`&Path`, `String`), never byte buffers.

- `process_sources(source_paths: &[PathBuf], …)` — good.
- ~~`process_sources(bytes: Vec<u8>, …)`~~ — forbidden. Frontend would have had to read a PDF
  into memory and base64-encode it across the IPC boundary, which is exactly the
  "memory mirage" Rule 1 forbids.

Result images go to a temp dir under the desktop app's `cache_dir`. Path strings are
returned via the IPC layer; the frontend loads them through `asset://localhost/`.

### Rule 2 — Progress reporting

Long-running work reports progress via `tokio::sync::mpsc::Sender<TaskProgress>`. The
desktop app forwards each message to the frontend with `app.emit("task-progress", &msg)`.

The `TaskProgress` shape (defined in `shalgalt_core::domain::progress` and re-exported
here) is:

```rust
pub struct TaskProgress {
    pub task_id: String,
    pub processed: usize,
    pub total: usize,
    pub stage: TaskStage,   // serializes as snake_case
    pub message: Option<String>,
}
```

`TaskStage` variants — keep the enum exhaustive, do not add a wildcard arm:

| Stage               | Emitted at                                                  |
| ------------------- | ----------------------------------------------------------- |
| `loading_pdf`       | After opening the PDF, before rasterization.                |
| `rasterizing`       | Per-page rasterization loop.                                |
| `detecting_markers` | ArUco corner detection.                                     |
| `reading_bubbles`   | Per-bubble fill-ratio sweep.                                |
| `grading`           | After all sheets are read, before scoring.                  |
| `saving`            | While the frontend persists results into SQLite.            |
| `done`              | Terminal — pipeline succeeded.                              |
| `failed`            | Terminal — pipeline aborted; `message` carries the reason.  |

Callers MUST emit at least one `done` or `failed` stage so the frontend's
`progress.svelte.ts` store can release the spinner.

## V1.0 Algorithmic Decisions (master plan §6.5)

These are **locked**. Re-litigation requires an ADR under `docs/adr/`.

- **Markers**: ArUco `DICT_6X6_50`. The four corner squares from P0/P1 are removed.
  Marker IDs: top-left `0`, top-right `1`, bottom-right `2`, bottom-left `3`.
- **Alignment**: each detected marker contributes its four corners as homography
  correspondences; the page aligns whenever **≥3 of 4** markers are found (`MIN_MARKERS`).
  Fewer than 3 is a `BadRequest`. See ADR 0013.
- **Thresholding**: adaptive (Gaussian, block size 35–51, C ≈ 7) — not Otsu — because
  schools photocopy sheets and the global histogram is unreliable.
- **Deskew**: marker-driven. After ArUco detection, fit the homography from marker corners
  with `find_homography` (RANSAC) and `warpPerspective` once per page. No rotation-only
  fallback.
- **Confidence**: per bubble, fill ratio in `[0, 1]`. Decision rule:
  - `ratio < 0.35` → unfilled.
  - `ratio > 0.65` → filled.
  - `ratio ∈ [0.35, 0.65]` → uncertain. Any sheet with at least one uncertain bubble is
    flagged `needs_review` and surfaces in the manual-review queue (P3-05).

## Testing

```bash
cargo test -p shalgalt-cv                       # unit + integration
cargo test -p shalgalt-cv --test pdfium_marker_raster
```

The marker-raster regression test depends on `shalgalt-pdf` as a `dev-dependency` to
generate the input PDF, then pipes it through pdfium-render. Keeping pdfium as
dev-only on `shalgalt-pdf`'s side avoids a cycle in the runtime dependency graph.

## What does NOT belong here

- Database access. Results flow back via `process_pdf`'s return value or progress events;
  the frontend persists them through `tauri-plugin-sql`.
- Tauri / IPC types. The IPC payload shapes live in `shalgalt-core::domain::progress`.
- The grading engine. `(template, parsed_sheet, answer_key) -> graded_sheet` lives in
  `shalgalt-core::grading`. CV stops at "parsed sheet".
