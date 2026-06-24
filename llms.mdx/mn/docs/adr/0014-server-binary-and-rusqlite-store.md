# ADR 0014 — Standalone server & rusqlite store (https://filename24.github.io/shalgalt-omr/mn/docs/adr/0014-server-binary-and-rusqlite-store)



<Callout type="success" title="Accepted · 2026-06-17">
  **Deciders:** filename24 · **Related issues:** P5-05 (`apps/server` binary), P5-03/04 (the
  endpoints it serves) · **Related ADRs:** ADR 0013 (the `DataStore` seam + DTOs), ADR 0003
  (workspace boundaries)
</Callout>

## Context [#context]

ADR 0013 leaves persistence to a host-injected `DataStore`. Two hosts need one:
`apps/server` (a headless LAN deployment) and `apps/desktop` (the embedded API). The
desktop is the hard case: its SQLite file is owned by `tauri-plugin-sql&#x60; (sqlx), and the
repo rule is &#x2A;*"no DB writes in Rust — plugin-sql is the single writer"**
(`apps/desktop/CLAUDE.md`). Core must stay database-free (§8.2), so the rusqlite code cannot
live there.

Where should the concrete store live, and how does the desktop read SQLite without becoming
a second writer?

## Options considered [#options-considered]

| Question                           | Option                           | Verdict                                                                                         |
| ---------------------------------- | -------------------------------- | ----------------------------------------------------------------------------------------------- |
| Where does the rusqlite impl live? | In core behind a feature         | **Rejected** — violates §8.2's "core depends only on serde/thiserror/anyhow/axum/tower-http".   |
|                                    | Duplicated in each host          | **Rejected** — DRY; \~250 lines of SQL drift between two binaries.                              |
|                                    | A shared `crates/shalgalt-store` | **Chosen** — one impl, both hosts depend on it, core stays clean.                               |
| How does the desktop read?         | A read-write rusqlite connection | **Rejected** — two writers on one file; breaks the single-writer rule.                          |
|                                    | A read-only rusqlite connection  | **Chosen** — `SQLITE_OPEN_READ_ONLY`; cannot write, so plugin-sql stays the only writer.        |
| Server CLI vs env                  | All env vars                     | **Rejected** — master plan §6.6 specifies `--bind`/`--db`/`--allow-origin`.                     |
|                                    | CLI flags + token in env         | **Chosen** — flags for config, `SHALGALT_API_TOKEN` in env (a secret belongs in env, not argv). |

## Decision [#decision]

<Callout type="info" title="Decision">
  Introduce a shared **`crates/shalgalt-store`** crate exporting a `rusqlite`-backed
  `SqliteStore` (read/write, server) plus a `DeferredReadOnlyStore` (read-only, desktop), so
  one store implementation serves both hosts while core stays database-free and plugin-sql
  remains the single writer.
</Callout>

**1. New crate `crates/shalgalt-store`.** It depends on `shalgalt-core` (for the trait +
DTOs) and `rusqlite` (`bundled`), and exports `SqliteStore` implementing `DataStore`, plus
a `DeferredReadOnlyStore` for the desktop. Workspace §8.1/§8.2 is updated to list it.

**2. Two postures.**

* `open_read_only(path)` — desktop. Opens `SQLITE_OPEN_READ_ONLY`; `create_*` returns the
  core read-only error. The API read path is **separate from the plugin-sql write
  ownership**: plugin-sql remains the single source of truth for writes, the API only reads.
* `open_read_write(path)` + `migrate()` — server. Owns the whole file (WAL), serves the full
  read/write surface.

**3. The desktop reader is deferred.** `tauri-plugin-sql` creates the SQLite file lazily (on
the frontend's first `Database.load`), which is *after* the embedded server boots.
`DeferredReadOnlyStore` opens a fresh read-only connection per request and reports an empty
database until the file exists, so boot never fails on a missing file. Traffic is local and
low-volume, so per-request connection setup is acceptable.

**4. One schema source of truth.** `migrate()` applies the desktop's `migrations/*.sql`,
embedded by relative path via `include_str!`. It is bootstrap-only (no-op if `templates`
already exists), so the server can stand up a fresh database with the exact DDL the desktop
uses, while schema *evolution* on a shared file stays owned by plugin-sql's migration chain.

**5. SQLite linkage.** `rusqlite`'s `bundled` feature links the same vendored SQLite that
sqlx (`libsqlite3-sys`) already pulls into the desktop binary; cargo unifies them into one
SQLite rather than two.

**6. Server assembly is a library.** `apps/server` is split `lib.rs` (CLI + `build_app` +
`serve`) / `main.rs` so the auth path is integration-testable via `tower::oneshot` without
binding a socket. Layer order: auth inside CORS, so the CORS preflight (`OPTIONS`, no
`Authorization`) is answered before the bearer guard.

## Consequences [#consequences]

* The same `/v1/` data is served inside Tauri and from the server (P5 acceptance met), with
  one store implementation.
* The desktop's "no DB writes in Rust" rule holds: the carve-out is a *pure reader*, all SQL
  confined to `shalgalt-store`. `apps/desktop/AGENTS.md` records the carve-out.
* `AppDirs::db_path()` was corrected to resolve against `app_config_dir` — where
  `tauri-plugin-sql` actually writes — not `app_data_dir` (the prior "reference only" value
  was wrong and never exercised).
* Two SQLite access layers (sqlx writer, rusqlite reader) coexist; readers never block the
  writer in SQLite, and a 5 s `busy_timeout` absorbs brief contention.

## Follow-up [#follow-up]

* A bundled Release asset for `shalgalt-server` lands in P6 (`release.yml`).
* Token rotation for server mode is documented but not yet automated.
* **Blocking I/O on the async executor.** `DataStore` methods are synchronous and run
  directly inside the axum handlers, and `SqliteStore` serializes them through one
  `Mutex<Connection>`. For the desktop (single user) and a small-school LAN server (low
  concurrency, WAL reads) this is acceptable. If concurrency grows, offload the calls with
  `tokio::task::spawn_blocking` (or make `DataStore` async) and/or use a connection pool so
  a slow query cannot park a worker thread. Deferred until measured load justifies it.
* **Pagination.** List endpoints are currently unbounded (see ADR 0013 — the desktop
  browser paginates client-side). A server-enforced page-size cap on `/v1/results` lands
  when result volumes warrant it.

<Cards>
  <Card href="/adr/0013-rest-api-v1-datastore" title="ADR 0013 — REST `/v1` & DataStore seam">
    The `DataStore` seam and DTOs this crate implements, and the versioned route surface.
  </Card>
  <Card href="/adr/0003-cargo-workspace-and-crate-boundaries" title="ADR 0003 — Cargo workspace & crate boundaries">
    The workspace rules (§8.1/§8.2) that keep the rusqlite code out of core.
  </Card>
</Cards>
