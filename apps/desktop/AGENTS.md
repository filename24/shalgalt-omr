# `apps/desktop` — Tauri Shell

The native desktop application. This crate is the only place where Tauri-bound concerns
live — IPC commands, plugin registration, the SQL plugin host, the axum server harness,
OS-aware path resolution, and the application bootstrap. Every Tauri-agnostic piece
(domain, grading, axum router, CV pipeline, PDF renderer) lives in `crates/`.

> Repo-level rules, language conventions, and locked decisions live in
> [`/AGENTS.md`](../../AGENTS.md). This file describes only what is specific to this crate.

## Module Layout

```
src/
  api/            — Rule 4 harness: tokio::spawn task + oneshot shutdown for axum.
                    Router itself comes from `shalgalt_core::api`.
  commands/       — `tauri::command` thin shims (scan, export, pdf). NO db CRUD.
  paths.rs        — OS-aware data/cache/scans dirs (Tauri `PathResolver`-backed).
  state.rs        — `AppState { dirs }`. No DB pool — plugin-sql owns the connection.
  lib.rs          — Bootstrap order:
                    tracing → AppDirs → plugin-sql(migrations) → axum → builder.
  main.rs         — `pub fn main() { shalgalt_omr_lib::run() }`.
migrations/       — SQL files embedded into Rust via `include_str!` and registered with
                    `tauri_plugin_sql::Migration { version, sql, kind: Up }`.
capabilities/     — Tauri 2 capability declarations.
```

## Bootstrap Order (Rule 4 + ARCHITECTURE §4)

`lib.rs::run()` strictly follows this order — moving any step earlier or later breaks one
of the critical rules:

1. `init_tracing()` — `tracing-subscriber` with `default-features = false` so it does
   **not** install a `log::Log` implementation. `tauri-plugin-log` owns that slot;
   double-installation panics.
2. Build the `Migration` vec from the embedded SQL files.
3. `tauri::Builder::default()`.
4. `tauri-plugin-single-instance` — must be the first plugin registered. Two processes
   on the same SQLite file corrupt state; this plugin focuses the existing window
   instead of launching a duplicate.
5. `tauri-plugin-window-state` (desktop only) — restore last window size/position.
6. `tauri-plugin-log` with rotating file target → `app_log_dir/shalgalt-omr*.log`.
7. `tauri-plugin-opener`, `tauri-plugin-dialog`, `tauri-plugin-fs`.
8. `tauri-plugin-sql` with the migrations vec attached to the `sqlite:shalgalt-omr.sqlite`
   URL.
9. `setup` callback spawns `bootstrap(handle)` on `tauri::async_runtime` — resolves
   `AppDirs`, prunes old PDF previews, then `api::spawn().await` and `app.manage(...)`
   the resulting `ApiHandle` + `AppState`.
10. `invoke_handler` registers the `tauri::generate_handler!` macro list.
11. `run(tauri::generate_context!())`.

## Critical Rules — desktop application

### Rule 1 — IPC Memory Mirage Avoidance

This crate is the **only** boundary that processes large user files. Every command in
`commands/*.rs` MUST take a path string, never bytes:

- New commands: `pdf_path: String`, `image_path: String`, `output_path: String`.
- **Forbidden** in command signatures: `bytes: Vec<u8>`, `data: String` (base64),
  `image: ImageBuffer<…>`, anything pre-decoded by the frontend.
- Result images go to a temp dir under `cache_dir`. The frontend loads them via the
  `asset://localhost/` protocol (declared in `capabilities/`), never `readFile → base64`.
- Frontend file system access policy is the corollary:
  - `tauri-plugin-dialog` for picking paths (PDF, xlsx-save, template JSON).
  - `tauri-plugin-fs` ONLY for small text payloads (CSV student rosters, `OmrTemplate`
    JSON import/export, `readDir` on the scans folder).
  - Never push PDFs / PNGs / any binary > ~1 MB through `fs.readFile`.

### Rule 2 — No UI Freeze on Batch Work

- All heavy work (CV, PDF generation, xlsx export) happens inside `tokio::spawn`, **not**
  inside the awaiting command body. The command returns a `task_id` string immediately
  and the spawned task drives the work.
- Progress events use `app.emit("task-progress", &TaskProgress { task_id, processed,
  total, stage, message })`. Frontend `progress.svelte.ts` is the single subscriber.
- Stage strings are snake_case and MUST match `shalgalt_core::domain::progress::TaskStage`:
  `loading_pdf`, `rasterizing`, `detecting_markers`, `reading_bubbles`, `grading`,
  `saving`, `done`, `failed`.

### Rule 4 — axum Server Independence

- `api::spawn(db_path)` returns an `ApiHandle` holding the oneshot shutdown sender. The
  handle is `app.manage(...)`-ed so the server is shut down gracefully when the Tauri app
  exits.
- The router itself is built in `shalgalt_core::api::router(state)` — this crate only owns
  the spawn / port-binding / shutdown wiring plus the injected `DataStore`. `apps/server`
  (P5) reuses the same router with a read-write store.
- CORS stays permissive (`Any`) here because the socket is bound to `127.0.0.1` and only
  the local webview can reach it. The allow-list + bearer auth posture is for `apps/server`
  (Server mode, master plan §6.6), layered there, not here.

## Database Access Policy

**No DB code in this crate.** `tauri-plugin-sql` owns the SQLite connection; the frontend
talks to it via `@tauri-apps/plugin-sql`. Rust commands NEVER call into the DB.

If a Rust task produces data that needs to land in SQLite (e.g., grading results), it
emits an event and the frontend persists it. This keeps backend/frontend ownership
boundaries clean and removes the question "which side has the up-to-date row?".

**One carve-out (P5):** the embedded HTTP API reads SQLite through a *read-only*
`shalgalt_store::DeferredReadOnlyStore` opened on `AppDirs::db_path()` (which resolves to
`app_config_dir` — where `tauri-plugin-sql` actually writes, not `app_data_dir`). It opens
`SQLITE_OPEN_READ_ONLY` and never issues a write, so plugin-sql remains the single writer
and the "which side has the up-to-date row?" question is untouched. This is a pure reader,
not "DB code": all SQL lives in `crates/shalgalt-store`. See ADR 0014 for the rationale.

### Migrations

- Files live in `migrations/` and are embedded at compile time via `include_str!` from
  `lib.rs`.
- To add one: drop a new file (e.g. `0003_topic.sql`), then append a `Migration { version,
  sql, kind: Up }` entry inside `run()`. Versions are monotonically increasing and never
  rewritten.
- Down migrations are not used; recovery is via export → fresh DB.

## Commands Surface

| Command                              | Where it lives           | Notes                                 |
| ------------------------------------ | ------------------------ | ------------------------------------- |
| `scan_grade_pdf`                     | `commands/scan.rs`       | Long-running. Emits `task-progress`.  |
| `rasterize_pdf_first_page`           | `commands/scan.rs`       | Editor backdrop. Returns image path.  |
| `pdf_generate_omr`                   | `commands/pdf.rs`        | `shalgalt-pdf` → file on disk.        |
| `pdf_render_template_preview`        | `commands/pdf.rs`        | Cached preview via SHA-256 of template.|
| `export_results_xlsx`                | `commands/export.rs`     | `shalgalt-core::export::xlsx`.        |

Every command body is a thin shim: validate input → delegate to a workspace crate →
convert anything that escapes to `AppError`. No business logic lives here.

## Common Commands

| Command            | Purpose                                                       |
| ------------------ | ------------------------------------------------------------- |
| `pnpm tauri dev`   | Native window + Rust core (auto-runs Vite). The default flow. |
| `pnpm tauri build` | Production native bundle.                                     |
| `cargo check -p shalgalt-omr` | Type-check this crate without invoking Tauri's webview build. |
