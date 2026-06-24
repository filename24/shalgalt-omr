# ADR 0004 — TypeScript bindings via ts-rs (https://filename24.github.io/shalgalt-omr/en/docs/adr/0004-ts-rs-type-codegen)



<Callout type="success" title="Accepted · 2026-05-08">
  **Deciders:** filename24 · **Implements:** P2-04
</Callout>

## Context [#context]

`OmrTemplate` and the IPC payloads cross the Rust/TS boundary every time the editor
saves or loads a template, every time a scan emits progress, and every time results
flow through axum. P0 maintained both halves by hand: a Rust struct with `serde`
attributes and a parallel TypeScript interface. Two fields renamed in `template.rs`
without updating `template.ts` would silently corrupt rows on read because the Zod
schema would still accept the unknown shape.

## Options considered [#options-considered]

| Option                                          | Wire-format parity      | Toolchain weight         | DX                                     |
| ----------------------------------------------- | ----------------------- | ------------------------ | -------------------------------------- |
| Hand-maintain TS interfaces (status quo)        | Manual, easy to drift   | Zero                     | Poor — silent breakage                 |
| Generate via OpenAPI / utoipa                   | Schema-grade parity     | Heavy — needs HTTP doc   | Overkill for IPC                       |
| **`ts-rs` derives running inside `cargo test`** | **Wire-format parity**  | **Light — one dep**      | **Best — runs in workspace test pass** |
| Generate via `specta` + `tauri-specta`          | Tight Tauri integration | Couples codegen to Tauri | Loses `apps/server` reuse              |

## Decision [#decision]

<Callout type="info" title="Decision">
  Adopt **`ts-rs = "10"` on `shalgalt-core`** — domain types derive `TS` and emit into
  `src/lib/types/generated/`.
</Callout>

CI runs `cargo test -p shalgalt-core --quiet` followed by
`git diff --exit-code -- src/lib/types/generated`, so a Rust-side schema change that
forgets re-generation fails the build.

Conventions:

* Hand-authored helpers (Zod schemas, `createEmptyTemplate`, `TemplateSummary`) live
  in `src/lib/types/template.ts` and re-export the generated types.
* `#[ts(optional)]` is reserved for `Option<T>` fields. For `Option`-equivalent
  patterns expressed via `serde(default, skip_serializing_if = …)` (e.g.
  `MarkerKind`), the Zod schema marks the field optional too — the `kind` field on
  `Marker` ships a `discriminatedUnion` schema with `.optional()` (P2-01 follow-up).
* `i64` IDs widen to `#[ts(type = "number")]` because SQLite IDs always fit
  `Number.MAX_SAFE_INTEGER` for our access patterns.

## Consequences [#consequences]

**Positive**

* Renames, additions, and field-type changes are caught by CI before they hit a
  template-load corruption path.
* `pnpm generate-types` (alias to `cargo test -p shalgalt-core --quiet`) gives the
  frontend devs a one-command refresh.

**Negative**

* ts-rs prints a non-fatal warning for `#[serde(skip_serializing_if = …)]`. Cosmetic
  only — emitted output is correct.
* `export_to` paths resolve relative to one directory above the manifest dir, so
  workspace-member crates use `../../../src/lib/types/generated/`. Documented inline
  in the derive macros.

## Follow-up [#follow-up]

* A future `prebuild` hook on `apps/web` could run the codegen automatically before
  `pnpm dev` starts; not pressing while the test-driven check covers CI.
* `MarkerKind` ergonomics: if the hand-authored Zod `.optional()` proves too easy to
  forget, we can move toward generated Zod (`zodios`-style) — out of scope for P2.
