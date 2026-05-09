# `crates/shalgalt-core` — Pure-Rust Core

Domain models, grading engine, axum router, xlsx export, and the `AppError` envelope.
Shared by `apps/desktop` and `apps/server` (P5). No `tauri`, no OpenCV, no pdfium on the
dependency path.

> Repo-level rules, language conventions, and locked decisions live in
> [`/AGENTS.md`](../../AGENTS.md). This file describes only what is specific to this crate.

## Module Layout

```
src/
  domain/         — Pure models. Depends on nothing. Rule 3 serialization unit.
    template.rs   — `OmrTemplate`, `BubbleGroup`, `Marker`, `BubbleKind` …
    paper.rs      — `PaperSpec`, `Orientation` (A4 portrait/landscape).
    progress.rs   — `TaskProgress`, `TaskStage`. Rule 2 IPC payload shape.
    result.rs     — `GradedSheet`, `BubbleReading`, confidence, needs_review.
    student.rs    — Roster row.
    mod.rs        — Re-exports.
  grading/        — Pure scoring engine: (OmrTemplate, ParsedSheet) -> GradedSheet.
  export/         — rust_xlsxwriter output for results.
  api/            — axum `Router` builder + CORS layer. Tauri-bound spawn lives in
                    `apps/desktop/src/api`; standalone harness is `apps/server`.
  error.rs        — `AppError` / `AppResult`. Implements `Serialize` for IPC payloads.
  lib.rs          — Module re-exports only. Read it first.
```

## Critical Rules — core

### Rule 3 — Template Serialization Format (canonical home)

This crate is the **single source of truth** for the `OmrTemplate` shape. The visual
editor's output (4 corner markers + bubble groups + answer key) serializes as JSON and
lives in `templates.json_schema` (TEXT). Never split into multiple columns or denormalize.

- Rust struct: `src/domain/template.rs`.
- TypeScript interface: `src/lib/types/template.ts` in the SvelteKit frontend.
- The two MUST stay in sync — change both together. P2-04 wires up `ts-rs` so the TS file
  is regenerated from the Rust derive on `cargo test -p shalgalt-core`. Do not hand-edit
  files in `src/lib/types/generated/`.
- Coordinates are normalized (`0.0–1.0`) so the same template works across DPIs and page
  sizes. Pixel-space coordinates are forbidden in `OmrTemplate`.

### Rule 4 — axum Router

The `Router` and CORS builder live in `api/`. They are agnostic to who spawns them — the
desktop app wraps them in a `tokio::spawn` + oneshot harness (Rule 4 lifecycle), and
`apps/server` (P5) runs them as the main task of the process.

What this means for code in this crate:

- Never call `axum::serve` here.
- Never bind a port here.
- Never read environment variables here. Auth + CORS-allow-list configuration is passed
  in by the host app.
- Routes are versioned under `/v1/`. Breaking changes require a bump and an ADR under
  `docs/adr/`.

## `AppError` envelope

`error.rs` defines `AppError` and `AppResult<T> = Result<T, AppError>`. Key invariants:

- `AppError` derives `Serialize` so it crosses the Tauri IPC boundary as JSON.
- `AppError::code` is a stable English identifier (e.g. `"pdf.parse_failed"`). It is NOT
  user-visible. The frontend maps `code` → Mongolian message in the P1 string-table.
- Free-form strings inside `AppError` (e.g. paths, underlying error text) are for logs
  and developer diagnostics, never for direct display.

## TS-RS Bindings (P2-04)

Every domain model that crosses IPC must derive `TS`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../src/lib/types/generated/")]
pub struct OmrTemplate { … }
```

`cargo test -p shalgalt-core` writes the `.ts` files into the SvelteKit `src/lib/types/
generated/` directory. The frontend script `pnpm generate-types` wraps the same command.
CI runs both jobs, and a drift between Rust and TS will fail `pnpm check`.

## Testing

Integration tests live in `tests/` (each file = separate binary, per Rust convention):

- `template_serde.rs` — golden-file round-trip for `OmrTemplate`. Catches accidental
  schema breakage (Rule 3 protection).
- `tests/fixtures/` — static JSON used by the round-trip test.

```bash
cargo test -p shalgalt-core            # unit + integration + ts-rs export
cargo test -p shalgalt-core --doc      # doc tests
```

## What does NOT belong here

- Any `tauri::*` import. Adding one would pollute `apps/server`'s dependency graph.
- pdfium, OpenCV, printpdf — those live in `shalgalt-cv` and `shalgalt-pdf`.
- Tokio spawn / port binding / env var reads — host apps own the process.
- DB code — there is no DB on the Rust side at all (`tauri-plugin-sql` owns SQLite).
