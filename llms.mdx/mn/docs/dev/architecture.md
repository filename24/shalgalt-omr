# Architecture (https://filename24.github.io/shalgalt-omr/mn/docs/dev/architecture)



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

<Callout type="warn" title="Load-bearing invariants">
  Every PR must respect these. They are the load-bearing invariants of the system — a change
  that violates any of them is rejected in review.
</Callout>

<Accordions>
  <Accordion title="Rule 1 — IPC carries path strings, never bytes">
    Tauri commands take `pdf_path: String` / `image_path: String`, never image or PDF
    buffers. Result images are written to disk and the frontend loads them through the
    `asset://localhost/` protocol. This avoids the "memory mirage" where multi-hundred-MB
    scans would be serialized across the IPC boundary. The `.shalgalt` reader/writer
    (`shalgalt-fileformat`) streams entries for the same reason — a project never lands in a
    single `Vec<u8>`.
  </Accordion>

  <Accordion title="Rule 2 — No UI freeze on batch work">
    Long-running work (grading a stack of sheets) runs in a `tokio::spawn`ed task. Progress
    is reported by emitting `task-progress` events with snake\_case stage names
    (`TaskProgress` / `TaskStage` in `shalgalt-core::domain::progress`). The command thread
    returns immediately.
  </Accordion>

  <Accordion title="Rule 3 — One template, one JSON column">
    `OmrTemplate` (markers + bubble groups + answer key) serializes as **a single JSON
    document** into one `TEXT` column (`templates.json_schema`). It is never split into
    multiple columns or denormalized. Coordinates are &#x2A;*normalized to `[0, 1]`** so a
    template works across DPIs and page sizes. The Rust struct
    (`shalgalt-core::domain::template`) and the TypeScript interface (`src/lib/types/`) are
    kept in sync by `ts-rs` codegen — never hand-edit the generated TS.
  </Accordion>

  <Accordion title="Rule 4 — The axum router never owns its lifecycle">
    The `Router` and CORS layer are built in `shalgalt-core::api`. That crate never calls
    `axum::serve`, never binds a port, and never reads environment variables. The host owns
    the lifecycle: `apps/desktop` wraps the router in a `tokio::spawn` + oneshot
    graceful-shutdown harness; `apps/server` runs it as the process's main task. Routes are
    versioned under `/v1/`; breaking changes require a version bump and an ADR.
  </Accordion>
</Accordions>

## Database ownership [#database-ownership]

All database access goes through `tauri-plugin-sql`. There is **no `sqlx` pool on the Rust
side**. Migrations are defined in Rust (`apps/desktop/src/lib.rs`, SQL embedded via
`include_str!`). The frontend reads and writes through `src/lib/db/*`. Rust commands never
touch the database: when a Rust task produces data that must be persisted (e.g. grading
results), it emits an event and the frontend writes it.

<Callout type="info" title="One exception">
  The standalone server uses a separate read-mostly `rusqlite` `DataStore` in
  `shalgalt-store` — see [ADR 0014](/adr/0014-server-binary-and-rusqlite-store).
</Callout>

## See also [#see-also]

<Cards>
  <Card href="/dev/workspace" title="Workspace map">
    What each crate and app owns.
  </Card>

  <Card href="/dev/api" title="HTTP API reference">
    The `/v1/` surface.
  </Card>

  <Card href="/adr" title="Architecture Decisions">
    The per-crate `AGENTS.md` files are authoritative for module-shape decisions; the ADRs
    record why the system looks the way it does.
  </Card>
</Cards>
