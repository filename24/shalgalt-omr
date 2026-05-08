# ADR 0003 — Cargo workspace + crate boundaries

- **Status**: Accepted
- **Date**: 2026-05-08
- **Deciders**: filename24
- **Implements**: P2-01 / P2-02 / P2-03

## Context

P0 shipped the entire Rust backend in a single crate at `src-tauri/`. The grading
core, the CV pipeline, and the Tauri shell all lived in the same dependency graph,
so `apps/server` (master plan §15, P5-05) — a headless build of the same grading
core for batch jobs — would either drag a `tauri` dependency into a daemon binary
or duplicate the domain types. Neither is acceptable.

## Options considered

| Option | Reusable core | Build cost | Surface area to refactor |
| ------ | ------------- | ---------- | ------------------------ |
| Stay single-crate, add a `headless` feature flag | Partial — `tauri` types still on the path via macros | Low | Low |
| Split into binary crates per app | Full | High — every shared type re-implemented | Highest |
| **Cargo workspace, library crates per concern** | **Full** | **Medium one-off** | **Bounded** |

## Decision

Convert the repository to a Cargo workspace with the following members:

```
apps/desktop/        Tauri 2 binary (IPC commands, axum spawn loop)
apps/server/         Headless axum binary (P5-05, scaffolded only)
crates/shalgalt-core/        Domain, grading, export, error envelope, axum router
crates/shalgalt-cv/          PDF rasterization, perspective, bubble density, preview
crates/shalgalt-pdf/         Pure-Rust OMR PDF renderer (printpdf — see ADR 0002)
crates/shalgalt-fileformat/  Reserved for `.shalgalt` zip + age (P4)
```

Constraints enforced by `cargo check`:

- `shalgalt-core` declares no `tauri` dependency.
- `shalgalt-cv` and `shalgalt-pdf` depend on `shalgalt-core` but not on `tauri`.
- `apps/desktop` is the only crate allowed to talk to `tauri::*`.
- The axum router lives in `shalgalt-core::api` so `apps/desktop` and `apps/server`
  mount the same builder.

## Consequences

**Positive**

- `apps/server` ships without dragging Tauri into a server binary.
- Future crates (e.g. `shalgalt-fileformat`, P4) follow a stable boundary pattern.
- Shared `[profile.release]` lives at the workspace root — every crate inherits the
  same LTO + strip settings.

**Negative**

- Compile times are slightly higher on the cold path (one extra link step per
  crate); cached incremental builds are unaffected.
- Path-based workspace deps require lockstep version bumps when we eventually
  publish (out of scope for v1.0).

## Follow-up

- ADR 0004 records the type-binding contract (ts-rs codegen) that follows naturally
  from the workspace shape.
- ADR 0005 records the IPC payload shape for the new `commands::pdf` module and how
  it stays inside the workspace's path-only Rule 1.
