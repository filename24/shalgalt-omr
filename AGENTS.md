# shalgalt-omr — Repository Guide

Local-First OMR Grading IDE. A Tauri 2 desktop app with a SvelteKit frontend and a Rust
core that grades scanned OMR PDFs offline using OpenCV, persists results in SQLite, and
exposes a background HTTP API for optional integrations.

> **Source of truth.** The v1.0 roadmap, locked architectural decisions, and the full
> sub-issue catalog live in
> [`.claude/PRPs/plans/shalgalt-omr-master.plan.md`](.claude/PRPs/plans/shalgalt-omr-master.plan.md).
> The corresponding GitHub tracking issue is [#11 — Master tracking issue](https://github.com/filename24/shalgalt-omr/issues/11).
> The original product spec still lives in [`docs/BLUEPRINT.md`](docs/BLUEPRINT.md) and
> the P0 module-shape doc in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md); both are
> superseded by the master plan where they conflict.

## Stack

| Layer        | Tech                                                                  |
| ------------ | --------------------------------------------------------------------- |
| Frontend     | SvelteKit 5 (adapter-static, SSG), Svelte 5 runes, TailwindCSS v4     |
| UI kit       | shadcn-svelte (bits-ui + tailwind-variants), @lucide/svelte icons     |
| Editor / UI  | svelte-konva (Canvas), paneforge (IDE panes)                          |
| Native shell | Tauri 2.0 + plugins: sql, dialog, fs, opener, log, single-instance, window-state |
| Core         | Rust — opencv-rust, pdfium-render, rust_xlsxwriter, printpdf          |
| Persistence  | `tauri-plugin-sql` + SQLite (single file in Tauri's `app_data_dir`)   |
| Bg API       | axum on `127.0.0.1:8080` (CORS permissive, oneshot graceful shutdown) |
| Async        | tokio (`full`)                                                        |

## Workspace Layout

```
docs/                           — BLUEPRINT, ARCHITECTURE, ADRs.
src/                            — SvelteKit frontend.
  routes/                       — App Router pages (dashboard, editor, grade, results).
  lib/
    db/                         — tauri-plugin-sql adapter (templates, results, students,
                                  getDb singleton).
    ipc/                        — Rust-only command wrappers (scan, export, pdf).
                                  NO db CRUD here.
    picker.ts                   — tauri-plugin-dialog wrappers (pickPdf etc.) — returns
                                  absolute paths only.
    stores/                     — Svelte 5 runes stores
                                  (progress / currentTemplate / session).
    components/                 — UI (shell, editor, grader, results).
      ui/                       — shadcn-svelte component copies (CLI-generated, freely
                                  editable).
    utils.ts                    — `cn()` Tailwind-merge helper + bits-ui type re-exports
                                  (shadcn contract).
    types/                      — Shapes mirroring Rust `domain` (template, result,
                                  progress). `generated/` is written by `ts-rs`.

apps/
  desktop/  ── AGENTS.md        — Tauri shell, IPC commands, axum harness, migrations.
  server/   ── AGENTS.md        — Standalone HTTP server (P5).

crates/
  shalgalt-core/         ── AGENTS.md   — Domain models, grading, axum router, AppError,
                                          xlsx export, DataStore seam. Pure Rust. Rule 3 home.
  shalgalt-store/        ── AGENTS.md   — rusqlite DataStore for the HTTP API (P5). The one
                                          place a second SQLite connection lives.
  shalgalt-pdf/          ── AGENTS.md   — printpdf-based OMR sheet generator with
                                          embedded Noto Sans + Noto Sans Mongolian.
  shalgalt-cv/           ── AGENTS.md   — pdfium + OpenCV pipeline; TaskProgress shape.
  shalgalt-fileformat/   ── AGENTS.md   — `.shalgalt` zip + age encryption (P4).
```

Each Cargo workspace member has its own `AGENTS.md` (with `CLAUDE.md` as a symlink) that
covers package-specific rules, module layout, and locked algorithmic decisions. Treat
those files as authoritative for anything inside that package.

## Common Commands

| Command            | Purpose                                                       |
| ------------------ | ------------------------------------------------------------- |
| `pnpm install`     | Install JS deps                                               |
| `pnpm dev`         | SvelteKit dev server only (browser, no Tauri webview)         |
| `pnpm tauri dev`   | Native window + Rust core (auto-runs Vite). The default flow. |
| `pnpm check`       | `svelte-kit sync` + `svelte-check`                            |
| `pnpm build`       | Build SvelteKit assets (consumed by `tauri build`)            |
| `pnpm tauri build` | Production native bundle                                      |
| `cargo check --workspace` | Type-check every Cargo member                          |
| `cargo test --workspace`  | Run all Rust tests (unit + integration + ts-rs export) |

## Critical Rules (from BLUEPRINT §3) — at a glance

These are **hard rules**; every PR must respect them. Detailed obligations live in the
package that enforces each rule — follow the link.

| Rule | Topic                              | Detailed in                                           |
| ---- | ---------------------------------- | ----------------------------------------------------- |
| 1    | IPC Memory Mirage Avoidance        | [`apps/desktop/AGENTS.md`](apps/desktop/AGENTS.md)    |
|      | (path strings, `asset://localhost/`)| with corollary in [`crates/shalgalt-cv`](crates/shalgalt-cv/AGENTS.md) and [`crates/shalgalt-fileformat`](crates/shalgalt-fileformat/AGENTS.md) |
| 2    | No UI Freeze on Batch Work         | [`apps/desktop/AGENTS.md`](apps/desktop/AGENTS.md)    |
|      | (`tokio::spawn`, `task-progress`)  | with payload shape in [`crates/shalgalt-cv`](crates/shalgalt-cv/AGENTS.md) |
| 3    | Template Serialization Format      | [`crates/shalgalt-core/AGENTS.md`](crates/shalgalt-core/AGENTS.md) |
|      | (`OmrTemplate` JSON, normalized coords)| with renderer side in [`crates/shalgalt-pdf`](crates/shalgalt-pdf/AGENTS.md) |
| 4    | axum Server Independence           | [`apps/desktop/AGENTS.md`](apps/desktop/AGENTS.md)    |
|      | (router lives in core, host owns spawn)| and [`apps/server/AGENTS.md`](apps/server/AGENTS.md) |

Quick summary so reviewers can spot violations without opening five files:

- **No bytes across IPC.** Commands take `pdf_path: String` / `image_path: String`. Result
  images go to disk; the frontend loads them via `asset://localhost/`.
- **No blocking work on the command thread.** Spawn a tokio task and emit
  `task-progress` events with snake_case stages.
- **No splitting `OmrTemplate` into columns.** It serializes as JSON into a single TEXT
  column. Coordinates are normalized to `[0, 1]`.
- **No `axum::serve` outside a host app.** The router is built in `shalgalt-core`;
  `apps/desktop` and `apps/server` own the lifecycle.

## UI Kit — shadcn-svelte

The frontend UI is built on **shadcn-svelte** (bits-ui + tailwind-variants). Components
are copied into the repo, not imported from a package — that is the whole point of the
shadcn model.

- **Location**: `src/lib/components/ui/<component>/*` — every file there is owned by us
  and freely editable. Treat them like first-party code.
- **Adding a component**: `pnpm dlx shadcn-svelte@latest add <name> --yes`. The CLI reads
  [`components.json`](components.json) and writes into `src/lib/components/ui/`.
- **Theme tokens**: see `src/app.css`. Two parallel families coexist — legacy `--color-*`
  tokens and the shadcn contract (`--background`, `--primary`, …). Both alias the same
  OKLCH palette. Add new tokens to *both* sides when introducing them.
- **`src/lib/utils.ts` is part of the shadcn contract**. It exports `cn()` plus the
  `WithElementRef` / `WithoutChild` / `WithoutChildrenOrChild` re-exports that the
  CLI-generated components import. Do not delete or rename these.
- **Toaster**: `<Toaster />` from `$lib/components/ui/sonner` is mounted once in the root
  layout. Use `import { toast } from "svelte-sonner"` everywhere else.
- **UI strings**: shadcn ships English defaults. Per the Mongolian-only UI rule, any
  user-visible copy added to a shadcn component (button labels, dialog headings, toast
  messages) must come from the P1 string-table. P0 placeholders can stay English with a
  `// P0 placeholder` comment.

## Database Access Policy

**All DB access is through `tauri-plugin-sql`.** There is no `sqlx` pool on the Rust side.

- Migrations: defined in Rust (`apps/desktop/src/lib.rs`) with SQL embedded via
  `include_str!`. Files live in [`apps/desktop/migrations/`](apps/desktop/migrations/).
  See [`apps/desktop/AGENTS.md`](apps/desktop/AGENTS.md) for the add-migration recipe.
- Frontend reads/writes via `$lib/db/*` modules. Never call `Database.load()` outside
  `$lib/db/index.ts` — the `getDb()` singleton caches the connection.
- Rust commands NEVER read or write the DB. If a Rust task produces data that needs to
  land in SQLite (e.g., grading results), it emits an event and the frontend persists
  it. This keeps backend/frontend ownership boundaries clean.

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
- Backend `AppError.code` stays English (it is a stable identifier, not user-visible).
  The frontend maps `code` → Mongolian message in the string table.
- i18n machinery (multi-language switcher) is **not** in scope. Mongolian is the only
  shipping language. Treat any framework choice (e.g. `svelte-i18n`) as overkill until P5.

## Branching & Commits

- `stable` — release branch. CI publishes from here. Do not commit directly.
- `develop` — integration branch. PRs from `feat/*` land here.
- `feat/<phase>-<topic>` — work branches off `develop`. One feature per PR.
- Conventional Commits: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `perf`, `ci`.
  Subject is one short imperative line; body explains the *why* if the diff doesn't.
- Never push `session/*` branches to remote.

## Continuous Integration

GitHub Actions runs on every push to `develop`/`stable` and on PRs targeting them.

| Job          | Purpose                                                              |
| ------------ | -------------------------------------------------------------------- |
| `web-check`  | `pnpm install --frozen-lockfile`, `svelte-kit sync`, `pnpm check`    |
| `rust-check` | apt deps (Tauri + OpenCV + clang) → `cargo fmt --check` → `clippy -D warnings` → `cargo check --workspace` |

Workflow file: [`.github/workflows/ci.yml`](.github/workflows/ci.yml). Both jobs must be
green before merge. Heavy bundling (signed installers, pdfium download, packaging) is
handled by the dedicated `release.yml` workflow that triggers on `v*` tags — landing in
P6 per master plan §10.

## Phased Roadmap (v1.0 target)

Aligned with master plan §10 and the GitHub milestones `v0.2.0` … `v1.0.0`.

| Phase | Milestone                  | Focus                                                                                          |
| ----- | -------------------------- | ---------------------------------------------------------------------------------------------- |
| P0    | (pre-roadmap)              | Foundation: scaffolding, plugin-sql, Rust modules, IDE shell — **complete**                    |
| P1    | (pre-roadmap)              | svelte-konva template editor + Mongolian-standard preset — **complete**                        |
| P2    | `v0.2.0 — Workspace + PDF` | Cargo workspace split, `shalgalt-pdf` crate, light theme + comfort mode, dashboard widgets — **current** |
| P3    | `v0.3.0 — Grading`         | `shalgalt-cv` (ArUco + adaptive threshold + confidence), grading engine, batch + manual review |
| P4    | `v0.4.0 — Project file`    | `.shalgalt` zip + `age` encryption, exam management, answer-key entry, file association        |
| P5    | `v0.5.0 — Excel + API`     | xlsx export, results browser, REST `/v1/`, `apps/server` standalone binary, OpenAPI            |
| P6    | `v0.6.0 — Distribution`    | `release.yml`, code signing, opt-in `tauri-plugin-updater`, AppImage + `.deb`                  |
| P7    | `v0.7.0 — Docs`            | Developer mdBook + Mongolian VitePress user manual + API reference + sample `.shalgalt` files  |
| P8    | `v0.8.0 — Hardening`       | A11y audit, performance pass, security review                                                  |
| P9    | `v1.0.0 — Public release`  | Final QA, signed bundles, public Release                                                       |

## Decisions Locked

Each item below is locked. Re-litigation requires an ADR under `docs/adr/` plus a master-
plan update in the same PR.

### Foundational (already in force)

- **OpenCV** is in use. System OpenCV is a build-time prerequisite (see README).
- **`tauri-plugin-sql`** owns the SQLite connection. No separate sqlx pool.
- **Code & docs language**: English only (see Language Conventions above).
- **UI language**: Mongolian only. No multi-language switcher; i18n machinery deferred to P5.
- **Distribution targets**: Windows (priority 1), macOS (2), Linux (3) — per master plan §6.9.

### v1.0-direction (locked by master plan §6, take effect during P2+)

- **Cargo workspace + monorepo** — `apps/desktop`, `apps/server`, `crates/shalgalt-{core,pdf,cv,fileformat}`. Migration completed in P2-01.
- **PDF generator**: `printpdf` (pure Rust, embedded Noto Sans + Noto Sans Mongolian). Lives in [`crates/shalgalt-pdf`](crates/shalgalt-pdf/AGENTS.md).
- **Project-file format**: `.shalgalt` zip container with optional `age` passphrase encryption. `manifest.json` always plaintext. See [`crates/shalgalt-fileformat`](crates/shalgalt-fileformat/AGENTS.md).
- **CV markers**: ArUco `DICT_6X6_50` replaces corner squares. Adaptive thresholding + auto-deskew + per-bubble confidence scoring; sheets with any bubble in `[0.35, 0.65]` ⇒ `needs_review`. Detail in [`crates/shalgalt-cv`](crates/shalgalt-cv/AGENTS.md).
- **HTTP API**: versioned under `/v1/`. Local mode = `127.0.0.1`, no auth. Server mode = `0.0.0.0`, bearer token from `SHALGALT_API_TOKEN`, CORS allow-list. Detail in [`apps/server`](apps/server/AGENTS.md).
- **Theming**: light theme is the default; dark theme optional via `mode-watcher`. No high-contrast mode. A "Comfortable" typography toggle (16 → 18 px base) sits next to the theme toggle.
- **Distribution channel**: GitHub Releases triggered by `v*` tags via `tauri-action`.
- **Auto-update**: `tauri-plugin-updater` with a static manifest on `gh-pages`. **Default OFF** so offline schools never see prompts.
- **No telemetry, no crash reporting, no analytics.** Local rotating logs only.

## Per-Package Docs

The repo-level guide is **this file plus the master plan**. Every Cargo workspace member
has its own `AGENTS.md` (with `CLAUDE.md` as a symlink); start there for code in that
package.

- [`AGENTS.md`](AGENTS.md) — repo rules, locked decisions, language conventions (this file).
- [`apps/desktop/AGENTS.md`](apps/desktop/AGENTS.md) — Tauri shell + IPC commands + axum harness + migrations.
- [`apps/server/AGENTS.md`](apps/server/AGENTS.md) — standalone HTTP server (P5).
- [`crates/shalgalt-core/AGENTS.md`](crates/shalgalt-core/AGENTS.md) — domain, grading, router, AppError, xlsx, DataStore (Rule 3 home).
- [`crates/shalgalt-store/AGENTS.md`](crates/shalgalt-store/AGENTS.md) — rusqlite DataStore for the HTTP API (P5).
- [`crates/shalgalt-pdf/AGENTS.md`](crates/shalgalt-pdf/AGENTS.md) — printpdf renderer + Noto fonts.
- [`crates/shalgalt-cv/AGENTS.md`](crates/shalgalt-cv/AGENTS.md) — pdfium + OpenCV pipeline + TaskProgress.
- [`crates/shalgalt-fileformat/AGENTS.md`](crates/shalgalt-fileformat/AGENTS.md) — `.shalgalt` zip + age (P4 placeholder).
- [`.claude/PRPs/plans/shalgalt-omr-master.plan.md`](.claude/PRPs/plans/shalgalt-omr-master.plan.md) — v1.0 architecture, roadmap, sub-issue catalog.
- [`.claude/PRPs/plans/issue-map.json`](.claude/PRPs/plans/issue-map.json) — `spec_id` ↔ GitHub issue number mapping.
- GitHub master tracking: [#11](https://github.com/filename24/shalgalt-omr/issues/11).

## Security & Secrets

- No secrets are committed. The local SQLite file lives in Tauri's bundle-identifier
  `app_data_dir` (e.g. `~/Library/Application Support/dev.filename.shalgalt-omr.app/`).
- The bundled axum server binds to `127.0.0.1` only. If exposing it (server mode), see
  [`apps/server/AGENTS.md`](apps/server/AGENTS.md) for the bearer-token + CORS allow-list
  requirements that land in P5.

## When Documentation Drifts

If anything here disagrees with `docs/ARCHITECTURE.md` or actual code, the code wins for
behavior and ARCHITECTURE wins for module-shape decisions. If the disagreement is between
this file and a per-package `AGENTS.md`, the package-level file wins for that package.
Update the relevant file in the same PR that introduces the drift.
