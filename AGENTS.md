# shalgalt-omr — Repository Guide

Local-First OMR Grading IDE. A Tauri 2 desktop app with a SvelteKit frontend and a Rust core
that grades scanned OMR PDFs offline using OpenCV, persists results in SQLite, and exposes a
background HTTP API for optional integrations.

> **Source of truth.** Product spec lives in [`docs/BLUEPRINT.md`](docs/BLUEPRINT.md), folder
> structure and module responsibilities in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Stack

| Layer        | Tech                                                                  |
| ------------ | --------------------------------------------------------------------- |
| Frontend     | SvelteKit 5 (adapter-static, SSG), Svelte 5 runes, TailwindCSS v4     |
| Editor / UI  | svelte-konva (Canvas), paneforge (IDE panes)                          |
| Native shell | Tauri 2.0                                                             |
| Core         | Rust — opencv-rust, pdfium-render, rust_xlsxwriter                    |
| Persistence  | `tauri-plugin-sql` + SQLite (single file in `ProjectDirs::data_dir`)  |
| Bg API       | axum on `127.0.0.1:8080` (CORS permissive, oneshot graceful shutdown) |
| Async        | tokio (`full`)                                                        |

## Workspace Layout

```
docs/             — BLUEPRINT, ARCHITECTURE, future ADRs
src/              — SvelteKit frontend
  routes/         — App Router pages (dashboard, editor, grade, results)
  lib/
    db/           — tauri-plugin-sql adapter (templates, results, students, getDb singleton)
    ipc/          — Rust-only command wrappers (scan, export). NO db CRUD here.
    stores/       — Svelte 5 runes stores (progress.svelte.ts subscribes to `task-progress`)
    components/   — UI (shell, editor, grader, results, ui)
    types/        — Shapes mirroring Rust `domain` (template, result, progress)
src-tauri/
  migrations/     — SQL files embedded into Rust via `include_str!` and registered with
                    `tauri_plugin_sql::Migration { version, sql, kind: Up }`
  src/
    domain/       — Pure models. Depends on nothing. Rule 3 serialization unit.
    scan/         — CV pipeline (OpenCV, pdfium). Heavy work; spawns tokio tasks.
    grading/      — Pure scoring engine: (OmrTemplate, ParsedSheet) -> GradedSheet.
    export/       — rust_xlsxwriter output.
    api/          — axum background server. Independent of Tauri window lifecycle.
    commands/     — `tauri::command` thin layer. ONLY Rust-only work (scan, export).
    error.rs      — `AppError` / `AppResult`. Implements `Serialize` for IPC-safe payloads.
    state.rs      — `AppState { dirs }`. No DB pool — plugin-sql owns the connection.
    paths.rs      — OS-aware data/cache/scans dirs (`directories::ProjectDirs`).
    lib.rs        — Bootstrap order: tracing → AppDirs → plugin-sql(migrations) → axum → builder.
```

## Common Commands

| Command            | Purpose                                                       |
| ------------------ | ------------------------------------------------------------- |
| `pnpm install`     | Install JS deps                                               |
| `pnpm dev`         | SvelteKit dev server only (browser, no Tauri webview)         |
| `pnpm tauri dev`   | Native window + Rust core (auto-runs Vite). The default flow. |
| `pnpm check`       | `svelte-kit sync` + `svelte-check`                            |
| `pnpm build`       | Build SvelteKit assets (consumed by `tauri build`)            |
| `pnpm tauri build` | Production native bundle                                      |

## Critical Rules (from BLUEPRINT §3)

These are **hard rules**; every PR must respect them.

### Rule 1 — IPC Memory Mirage Avoidance

- **Never** Base64-encode large image data across the IPC boundary. Frontend sends a
  **local absolute path string**; Rust writes result images to a temp dir and the frontend
  loads them via the `asset://localhost/` protocol.
- New Tauri commands that touch images MUST take `pdf_path: String` / `image_path: String`,
  never `bytes: Vec<u8>` or base64.

### Rule 2 — No UI Freeze on Batch Work

- All CV / DB-write work happens in `tokio::spawn`, not inside the awaiting command body.
- Progress is reported via `app.emit("task-progress", &TaskProgress { task_id, processed,
  total, stage, message })`. Frontend `progress.svelte.ts` is the single subscriber.
- The `task-progress` event payload shape MUST match `scan::pipeline::TaskProgress`
  (snake_case stages: `loading_pdf`, `rasterizing`, `detecting_markers`, `reading_bubbles`,
  `grading`, `saving`, `done`, `failed`).

### Rule 3 — Template Serialization Format

- The visual editor's output (4 corner markers + bubble groups + answer key) MUST serialize
  as `OmrTemplate` JSON and live in `templates.json_schema` (TEXT). Never split into
  multiple columns or denormalize into the DB.
- Coordinates are normalized (`0.0–1.0`) so the same template works across DPIs/page sizes.
- The Rust struct (`src-tauri/src/domain/template.rs`) and TS interface
  (`src/lib/types/template.ts`) must stay in sync — change both together.

### Rule 4 — axum Server Independence

- The HTTP server runs on its own tokio task spawned in `setup`. It does not block the Tauri
  UI thread, and it shuts down via `oneshot` channel held by `ApiHandle` in app state.
- CORS is currently permissive (`Any` origin/method/header). Tighten in P4 before any
  production release.

## Database Access Policy

**All DB access is through `tauri-plugin-sql`.** There is no `sqlx` pool on the Rust side.

- Migrations: defined in Rust (`lib.rs`) with SQL embedded via `include_str!`. To add one,
  drop a new file in `src-tauri/migrations/` and append a `Migration { version, sql, kind:
  Up }` entry. Versions are monotonically increasing.
- Frontend reads/writes via `$lib/db/*` modules. Never call `Database.load()` outside
  `$lib/db/index.ts` — the `getDb()` singleton caches the connection.
- Rust commands NEVER read or write the DB. If a Rust task produces data that needs to land
  in SQLite (e.g., grading results), it emits an event and the frontend persists it. This
  keeps backend/frontend ownership boundaries clean.

## Language Conventions (MANDATORY)

There are **two separate languages** used in this repository, and they do not overlap.

### 1. Code & Documentation — English only

Every comment, doc comment, identifier, log message, error string, commit message, PR
description, repo-level Markdown file, and inline TODO MUST be written in English.

- Rust `///` and `//` comments — English.
- TypeScript / Svelte `/** */` and `//` comments — English.
- Migration SQL comments — English.
- Files in `docs/`, `AGENTS.md`, `README.md`, ADRs — English.
- `tracing::info!` / `console.log` messages — English.
- `AppError` / thrown error messages — English.

When citing BLUEPRINT, refer to the rule number (e.g. "Rule 3 — Template Serialization")
rather than quoting Korean text.

> Code reviewers should treat any non-English string in source code or documentation as a
> blocker, except for the user-facing UI strings described next.

### 2. UI Language — Mongolian only

All end-user visible text in the application MUST be in Mongolian (Cyrillic script —
монгол хэл).

- Page titles, button labels, form labels, table headers — Mongolian.
- Validation messages, toast messages, dialog copy — Mongolian.
- Error surfaces shown to the user (translated from `AppError.code`) — Mongolian.
- Help text, tooltips, empty-state copy — Mongolian.

Implementation notes:

- A single string-table module (introduced in P1) is the source of truth. Components must
  not hard-code Mongolian strings inline; reference the table by key. This makes future
  re-translation cheap and keeps the rule mechanically enforceable.
- Backend `AppError.code` stays English (it is a stable identifier, not user-visible). The
  frontend maps `code` → Mongolian message in the string table.
- i18n machinery (multi-language switcher) is **not** in scope. Mongolian is the only
  shipping language. Treat any framework choice (e.g. `svelte-i18n`) as overkill until P5.

## Branching & Commits

- `stable` — release branch. CI publishes from here. Do not commit directly.
- `develop` — integration branch. PRs from `feat/*` land here.
- `feat/<phase>-<topic>` — work branches off `develop`. One feature per PR.
- Conventional Commits: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `perf`, `ci`.
  Subject is one short imperative line; body explains the *why* if the diff doesn't.
- Never push `session/*` branches to remote.

## Phased Roadmap (current target)

| Phase | Focus                                                                     |
| ----- | ------------------------------------------------------------------------- |
| P0    | Foundation: scaffolding, plugin-sql, Rust modules, IDE shell — **current** |
| P1    | svelte-konva template editor (markers + bubble groups)                    |
| P2    | CV pipeline: pdfium → 4-marker perspective → bubble density read          |
| P3    | Grading engine + result persistence + manual correction UI                |
| P4    | rust_xlsxwriter export + axum read-only endpoints                         |
| P5    | UX polish, i18n, packaging, autoupdater                                   |

## Decisions Locked

- **OpenCV** is in use. System OpenCV is a build-time prerequisite (see README).
- **`tauri-plugin-sql`** owns the SQLite connection. No separate sqlx pool.
- **Code & docs language**: English only (see Language Conventions above).
- **UI language**: Mongolian only. No multi-language switcher; i18n machinery deferred to P5.
- **macOS / Linux / Windows** are all supported targets.

## Per-Package Docs

When per-package guides become useful (e.g. once `apps/*` or workspace splits arrive),
each package gets its own `AGENTS.md` (with `CLAUDE.md` as a symlink). Until then, this
file is the only repo-level guide.

## Security & Secrets

- No secrets are committed. The local SQLite file lives in OS-specific
  `ProjectDirs::data_dir` (e.g. `~/Library/Application Support/dev.filename.shalgalt-omr/`).
- The bundled axum server binds to `127.0.0.1` only. If exposing it, harden CORS first.

## When Documentation Drifts

If anything here disagrees with `docs/ARCHITECTURE.md` or actual code, the code wins for
behavior and ARCHITECTURE wins for module-shape decisions. Update this file in the same PR
that introduces the drift.
