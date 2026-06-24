# Architecture (https://filename24.github.io/shalgalt-omr/en/docs/dev/architecture)



# Architecture [#architecture]

## Stack [#stack]

| Layer          | Technology                                                                                |
| -------------- | ----------------------------------------------------------------------------------------- |
| Frontend       | SvelteKit 5 (adapter-static, SSG), Svelte 5 runes, TailwindCSS v4                         |
| UI kit         | shadcn-svelte (bits-ui + tailwind-variants), `@lucide/svelte` icons                       |
| Editor / UI    | svelte-konva (Canvas), paneforge (IDE panes)                                              |
| Native shell   | Tauri 2.0 + plugins: sql, dialog, fs, opener, log, single-instance, window-state, updater |
| Core           | Rust — opencv-rust, pdfium-render, rust\_xlsxwriter, printpdf                             |
| Persistence    | `tauri-plugin-sql` + SQLite (a single file in Tauri's `app_data_dir`)                     |
| Background API | axum on `127.0.0.1:22345` (default; auto-fallback if busy)                                |
| Async          | tokio (`full`)                                                                            |

## High-level flow [#high-level-flow]

```text
          ┌──────────────────────────── SvelteKit frontend ───────────────────────────┐
          │  routes/ (dashboard, editor, grade, exams, review, results, settings)      │
          │  lib/db (tauri-plugin-sql)   lib/ipc (Rust command wrappers)   lib/i18n    │
          └───────────────┬───────────────────────────────────┬───────────────────────┘
                          │ tauri-plugin-sql                   │ Tauri IPC (path strings only)
                          ▼                                    ▼
                    ┌──────────┐                  ┌──────────────────────────────────┐
                    │  SQLite  │                  │  apps/desktop (Tauri host)        │
                    └──────────┘                  │   commands/  api harness  events  │
                                                  └───────────────┬───────────────────┘
                                                                  │ calls into pure crates
                       ┌──────────────────────────────────────────┼───────────────────────────┐
                       ▼                         ▼                 ▼                ▼
                shalgalt-core            shalgalt-cv        shalgalt-pdf     shalgalt-fileformat
              (domain, grading,        (pdfium + OpenCV    (printpdf OMR     (.shalgalt zip +
               axum router, xlsx,       pipeline,           sheet renderer)   age encryption)
               AppError, DataStore)     TaskProgress)
```

The desktop host (`apps/desktop`) is the only place that knows about Tauri. The four
`crates/` are pure Rust and are reused by the standalone server (`apps/server`).

## The four hard rules [#the-four-hard-rules]

Every PR must respect these. They are the load-bearing invariants of the system.

### Rule 1 — IPC carries path strings, never bytes [#rule-1--ipc-carries-path-strings-never-bytes]

Tauri commands take `pdf_path: String` / `image_path: String`, never image or PDF buffers.
Result images are written to disk and the frontend loads them through the
`asset://localhost/` protocol. This avoids the "memory mirage" where multi-hundred-MB scans
would be serialized across the IPC boundary. The `.shalgalt` reader/writer
(`shalgalt-fileformat`) streams entries for the same reason — a project never lands in a
single `Vec<u8>`.

### Rule 2 — No UI freeze on batch work [#rule-2--no-ui-freeze-on-batch-work]

Long-running work (grading a stack of sheets) runs in a `tokio::spawn`ed task. Progress is
reported by emitting `task-progress` events with snake\_case stage names
(`TaskProgress` / `TaskStage` in `shalgalt-core::domain::progress`). The command thread
returns immediately.

### Rule 3 — One template, one JSON column [#rule-3--one-template-one-json-column]

`OmrTemplate` (markers + bubble groups + answer key) serializes as **a single JSON document**
into one `TEXT` column (`templates.json_schema`). It is never split into multiple columns or
denormalized. Coordinates are &#x2A;*normalized to `[0, 1]`** so a template works across DPIs and
page sizes. The Rust struct (`shalgalt-core::domain::template`) and the TypeScript interface
(`src/lib/types/`) are kept in sync by `ts-rs` codegen — never hand-edit the generated TS.

### Rule 4 — The axum router never owns its lifecycle [#rule-4--the-axum-router-never-owns-its-lifecycle]

The `Router` and CORS layer are built in `shalgalt-core::api`. That crate never calls
`axum::serve`, never binds a port, and never reads environment variables. The host owns the
lifecycle: `apps/desktop` wraps the router in a `tokio::spawn` + oneshot graceful-shutdown
harness; `apps/server` runs it as the process's main task. Routes are versioned under
`/v1/`; breaking changes require a version bump and an ADR.

## Database ownership [#database-ownership]

All database access goes through `tauri-plugin-sql`. There is **no `sqlx` pool on the Rust
side**. Migrations are defined in Rust (`apps/desktop/src/lib.rs`, SQL embedded via
`include_str!`). The frontend reads and writes through `src/lib/db/*`. Rust commands never
touch the database: when a Rust task produces data that must be persisted (e.g. grading
results), it emits an event and the frontend writes it. The one exception is the standalone
server, which uses a separate read-mostly `rusqlite` `DataStore` in `shalgalt-store`
(see [ADR 0014](/dev/adr)).

## See also [#see-also]

* [Workspace map](/dev/workspace) — what each crate and app owns.
* [HTTP API reference](/dev/api) — the `/v1/` surface.
* The per-crate `AGENTS.md` files are authoritative for module-shape decisions.
