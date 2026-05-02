# shalgalt-omr — Initial Architecture (P0)

> Per BLUEPRINT.md §5 "First Task", this document defines the P0 folder structure and
> module responsibilities. Modules can be added or split further in later phases.

> **DB access policy (locked).** SQLite access is unified through
> [`tauri-plugin-sql`](https://v2.tauri.app/plugin/sql/). Migrations are registered on the
> Rust side as `Migration { version, sql, kind: Up }`, and the frontend reaches the database
> via `@tauri-apps/plugin-sql` using `Database.load("sqlite:shalgalt-omr.sqlite")`.
> The Rust side does NOT keep a separate `sqlx` pool. Therefore `commands/` contains no
> read/write commands; only **Rust-only heavy work** (CV / PDF / xlsx) is exposed through
> IPC.

## 1. Top-level Layout

```
shalgalt-omr/
├── docs/                          # design docs (BLUEPRINT, ARCHITECTURE, future ADRs)
├── src/                           # SvelteKit (SSG) frontend
├── src-tauri/                     # Rust core (Tauri 2.0)
├── static/                        # static assets
├── package.json
├── pnpm-lock.yaml
├── svelte.config.js
├── vite.config.js
├── tsconfig.json
├── components.json                # shadcn-svelte CLI config (added in P0)
└── (no tailwind.config / postcss.config — Tailwind v4 reads tokens from app.css `@theme`)
```

## 2. Frontend (`src/`)

```
src/
├── app.html
├── app.css                        # Tailwind v4 `@import` + `@theme inline` + design tokens
├── lib/
│   ├── utils.ts                   # cn() + bits-ui type re-exports (shadcn-svelte contract)
│   ├── ipc/                       # Rust-only command wrappers (scan, xlsx)
│   │   ├── index.ts
│   │   ├── scan.ts                # PDF split / grading invoke + emit listener
│   │   └── export.ts              # xlsx export invoke
│   ├── db/                        # tauri-plugin-sql repository layer
│   │   ├── index.ts               # cached Database.load (sqlite:shalgalt-omr.sqlite)
│   │   ├── templates.ts
│   │   ├── results.ts
│   │   └── students.ts
│   ├── stores/                    # Svelte 5 runes-based stores
│   │   ├── progress.svelte.ts     # subscribes to the task-progress emit
│   │   ├── currentTemplate.svelte.ts  # editor draft + pristine snapshot (P1+)
│   │   └── session.svelte.ts          # active job id, last PDF path, last template id
│   ├── hooks/
│   │   └── is-mobile/             # shadcn-svelte sidebar mobile breakpoint hook
│   ├── components/
│   │   ├── shell/                 # IDE shell — built on shadcn-svelte sidebar
│   │   │   ├── AppSidebar.svelte  # Sidebar.Root + Workspace nav (lucide icons)
│   │   │   └── StatusBar.svelte   # bottom strip — bound to progress + session stores
│   │   ├── editor/                # svelte-konva editor (P1)
│   │   │   ├── TemplateCanvas.svelte
│   │   │   ├── MarkerLayer.svelte         # 4 corner markers
│   │   │   ├── BubbleGroupLayer.svelte    # student-id / question bubble groups
│   │   │   ├── PropertyPanel.svelte
│   │   │   └── Toolbar.svelte
│   │   ├── grader/                # batch grading UI (P2/P3)
│   │   │   ├── PdfPicker.svelte
│   │   │   ├── ProgressOverlay.svelte
│   │   │   └── ReviewGrid.svelte          # manual correction for failed pages
│   │   ├── results/               # result browsing (P3/P4)
│   │   │   ├── ResultTable.svelte
│   │   │   └── ResultDetail.svelte
│   │   └── ui/                    # shadcn-svelte component copies (CLI-managed)
│   │       ├── button, badge, card, dialog, dropdown-menu,
│   │       ├── input, label, separator, sheet, sidebar,
│   │       ├── skeleton, sonner, tooltip
│   └── types/                     # shapes shared with the backend (manually defined)
│       ├── template.ts            # OmrTemplate, BubbleGroup, Marker
│       ├── result.ts
│       └── progress.ts
└── routes/                        # SvelteKit App Router
    ├── +layout.svelte             # Sidebar.Provider + AppSidebar + Inset header + StatusBar
    ├── +page.svelte               # dashboard (recent templates / results)
    ├── editor/
    │   └── +page.svelte           # template editor
    ├── grade/
    │   └── +page.svelte           # batch PDF grading
    └── results/
        └── +page.svelte           # result browsing / export
```

> **UI kit drift (resolved in P0).** The editor shell uses **shadcn-svelte sidebar** rather
> than a hand-rolled `IdeShell` + `ActivityBar` pair, and `components/ui/` is now CLI-managed
> shadcn copies instead of two bespoke atoms. paneforge stays in the dependency set for the
> P1 editor's resizable canvas / inspector splits.

## 3. Rust Core (`src-tauri/`)

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── build.rs
├── capabilities/
│   └── default.json
├── migrations/                    # SQL files embedded into Rust via `include_str!`
│   └── 0001_init.sql              # loaded in lib.rs
└── src/
    ├── main.rs                    # simply calls lib::run()
    ├── lib.rs                     # tauri::Builder + plugin-sql registration + setup hook
    ├── error.rs                   # AppError + AppResult — anyhow → frontend-safe
    ├── state.rs                   # AppState { dirs } (no DB pool — plugin-sql owns it)
    ├── paths.rs                   # AppDirs (data dir, scans dir, cache dir)
    ├── domain/                    # pure models (Rule 3 serialization unit)
    │   ├── mod.rs
    │   ├── template.rs            # OmrTemplate, BubbleGroup, Marker
    │   ├── student.rs
    │   └── result.rs
    ├── scan/                      # CV pipeline
    │   ├── mod.rs
    │   ├── pdf.rs                 # pdfium-render — PDF → Mat / DynamicImage
    │   ├── perspective.rs         # 4-corner detection + warpPerspective
    │   ├── bubbles.rs             # average-density bubble reading
    │   └── pipeline.rs            # tokio::spawn orchestrator, emit("task-progress")
    ├── grading/
    │   ├── mod.rs
    │   └── engine.rs              # template.json + parsed bubbles → score & detail
    ├── export/
    │   ├── mod.rs
    │   └── xlsx.rs                # rust_xlsxwriter
    ├── api/                       # axum background server (Rule 4 — independent task)
    │   ├── mod.rs
    │   ├── server.rs              # tokio::spawn on 8080, oneshot graceful shutdown
    │   ├── routes.rs              # GET /healthz, etc.
    │   └── cors.rs                # tower_http::cors policy
    └── commands/                  # tauri::command thin layer — IPC entry points
        ├── mod.rs
        ├── scan.rs                # arg: local absolute path (Rule 1)
        └── export.rs
```

## 4. Module Boundaries

| Layer              | Allowed dependencies                    | Notes                                                                                   |
| ------------------ | --------------------------------------- | --------------------------------------------------------------------------------------- |
| `domain`           | (none)                                  | Pure data. Depends on no infrastructure.                                                |
| `scan`             | `domain`                                | OpenCV / pdfium calls. Returns only `domain::ParsedSheet` types.                        |
| `grading`          | `domain`                                | Pure function: `(OmrTemplate, ParsedSheet) -> ScoreResult`. No CV.                      |
| `export`           | `domain`                                | xlsx serialization.                                                                     |
| `api`              | `domain`, `state`                       | Independent of the Tauri window. Receives only an `AppState` clone.                     |
| `commands`         | `domain`, `scan`, `grading`, `export`   | Thin shim: validate → call modules above → convert to `AppError`. No DB commands.       |
| `lib.rs`           | `commands`, `api`, `state`              | Boot order: tracing → AppDirs → plugin-sql(migrations) → axum spawn → tauri builder.    |
| Frontend `lib/db/` | `lib/types`                             | `tauri-plugin-sql` adapter. Owns SQL execution and `OmrTemplate` JSON serialization.    |

## 5. IPC & Threading Rules (Blueprint §3 applied)

- **Images travel as paths.** `commands::scan::grade_pdf` accepts `pdf_path: String` only.
  Result images are saved to `app_dirs.scans_dir` and the path is returned.
- **Heavy work is `tokio::spawn`.** A `tauri::command async fn` returns immediately after
  spawning and communicates via `app_handle.emit("task-progress", ...)`.
- **Standard progress payload** — `domain::TaskProgress { task_id, processed, total,
  stage }`.
- **The axum server** is spawned in the `setup` hook with `tokio::spawn`. On
  `AppHandle::on_window_event(CloseRequested)` a `oneshot` channel signals graceful
  shutdown.

## 6. SQLite Schema (P0)

`src-tauri/migrations/0001_init.sql`:

```sql
CREATE TABLE students (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  name            TEXT    NOT NULL,
  grade           INTEGER NOT NULL,
  class           INTEGER NOT NULL,
  roll_number     TEXT    NOT NULL,
  created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (grade, class, roll_number)
);

CREATE TABLE templates (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  title           TEXT    NOT NULL,
  json_schema     TEXT    NOT NULL,        -- serialized OmrTemplate (Rule 3)
  created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE results (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  student_id      INTEGER REFERENCES students(id) ON DELETE SET NULL,
  template_id     INTEGER NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
  total_score     REAL    NOT NULL,
  detail_answers  TEXT    NOT NULL,        -- JSON: per-question correct/wrong/blank
  image_path      TEXT,                    -- absolute path to the graded result image
  created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_results_template ON results(template_id);
CREATE INDEX idx_results_student  ON results(student_id);
```

## 7. Open Decisions (locked defaults below — change at any time)

| #   | Item                       | Default applied for P0                                                                          |
| --- | -------------------------- | ----------------------------------------------------------------------------------------------- |
| 1   | OpenCV dependency          | `opencv` crate with `clang-runtime`; build requires system OpenCV (`libopencv-dev`, etc.).     |
| 2   | pdfium dynamic library     | `pdfium-render` plus `bblanchon/pdfium-binaries` downloaded at install/build time.             |
| 3   | Target OSes                | Windows / macOS / Linux desktop.                                                               |
| 4   | UI language                | Mongolian only — string-table introduced in P1.                                                |
| 5   | Phase order                | P0 → P1 → P2 → P3 → P4 → P5.                                                                   |

P0 work proceeds with these defaults. Adjust as needed.
