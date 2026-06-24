# Workspace map (https://filename24.github.io/shalgalt-omr/mn/docs/dev/workspace)



# Workspace map [#workspace-map]

The repository is a Cargo workspace with a SvelteKit frontend at the root. Rust code lives
under `apps/` (host binaries) and `crates/` (pure libraries). A single `Cargo.lock` and
`target/` live at the workspace root.

```text
docs/                  — BLUEPRINT, ARCHITECTURE, ADRs, and this documentation site.
src/                   — SvelteKit frontend.
  routes/              — Pages: dashboard, editor, grade, exams, review, results, settings.
  lib/
    db/                — tauri-plugin-sql adapters (templates, results, students; getDb singleton).
    ipc/               — Rust-only command wrappers (scan, export, pdf). No DB CRUD here.
    i18n/              — Mongolian string table (single source of truth for UI copy).
    stores/            — Svelte 5 runes stores (progress / currentTemplate / session / comfort).
    components/        — UI (shell, editor, grader, review canvas, results); ui/ = shadcn copies.
    types/             — Shapes mirroring Rust domain; generated/ is written by ts-rs.

apps/
  desktop/             — Tauri shell: IPC commands, axum harness, migrations, event plumbing.
  server/              — Standalone HTTP server binary (shalgalt-server).

crates/
  shalgalt-core/       — Domain models, grading engine, axum router, AppError, xlsx export,
                         DataStore seam. Pure Rust. Home of Rule 3.
  shalgalt-store/      — rusqlite DataStore for the HTTP API. The one place a second SQLite
                         connection lives.
  shalgalt-pdf/        — printpdf-based OMR sheet generator (embedded Noto Sans + Noto Sans
                         Mongolian).
  shalgalt-cv/         — pdfium + OpenCV pipeline; emits TaskProgress.
  shalgalt-fileformat/ — .shalgalt zip container + age encryption.
```

## Crate responsibilities [#crate-responsibilities]

| Crate / app           | Owns                                                                | Must NOT contain                                          |
| --------------------- | ------------------------------------------------------------------- | --------------------------------------------------------- |
| `shalgalt-core`       | Domain types, grading, axum `Router`, `AppError`, xlsx, OpenAPI doc | `tauri`, OpenCV, pdfium, port binding, env reads, DB code |
| `shalgalt-cv`         | pdfium rasterization + OpenCV detection + confidence scoring        | Tauri, the axum router                                    |
| `shalgalt-pdf`        | printpdf rendering of a template to a printable sheet               | Tauri, OpenCV                                             |
| `shalgalt-fileformat` | Streaming `.shalgalt` read/write, age crypto wrapper                | The `OmrTemplate` struct (lives in core), Tauri           |
| `shalgalt-store`      | `rusqlite` `DataStore` impl for the server                          | The plugin-sql write path                                 |
| `apps/desktop`        | Tauri host: commands, migrations, axum spawn + shutdown harness     | Pure domain logic (delegate to crates)                    |
| `apps/server`         | Standalone process: CLI args, bind, auth, CORS allow-list           | Re-implementing the router (reuse core's)                 |

Each Cargo member has its own `AGENTS.md` (with `CLAUDE.md` as a symlink) describing
package-specific rules and locked algorithmic decisions. Treat those files as authoritative
for anything inside that package.

## Type generation (ts-rs) [#type-generation-ts-rs]

Domain models that cross the IPC boundary derive `ts_rs::TS` and export TypeScript
interfaces into `src/lib/types/generated/` when `cargo test -p shalgalt-core` runs. The
frontend script `pnpm generate-types` wraps the same command. A drift between the Rust struct
and the generated TS fails `pnpm check` in CI — this is the mechanical enforcement of
Rule 3.
