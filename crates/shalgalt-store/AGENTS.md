# `crates/shalgalt-store` — rusqlite DataStore for the HTTP API

The single `rusqlite`-backed implementation of `shalgalt_core::api::store::DataStore`,
shared by both hosts so the `/v1/` REST surface returns identical data inside Tauri and
from the standalone server (master plan §10 P5 acceptance).

> Repo-level rules, language conventions, and locked decisions live in
> [`/AGENTS.md`](../../AGENTS.md). This file describes only what is specific to this crate.

## Why this crate exists

`shalgalt-core` is deliberately database-free (see
[`crates/shalgalt-core/CLAUDE.md`](../shalgalt-core/CLAUDE.md)); it owns the `DataStore`
trait and the DTOs but never a connection. Rather than duplicate the SQL in `apps/desktop`
and `apps/server`, the concrete reader/writer lives here once and both hosts depend on it.

## Two postures

| Constructor | Host | Mode | Writes |
| --- | --- | --- | --- |
| `open_read_only(path)` | `apps/desktop` | `SQLITE_OPEN_READ_ONLY` | rejected with the core read-only error |
| `open_read_write(path)` + `migrate()` | `apps/server` | read-write, WAL | allowed |

The desktop opens the *same* SQLite file `tauri-plugin-sql` writes, read-only, so the
embedded API can never race the plugin-sql writer — plugin-sql stays the single source of
truth for writes (the desktop's "no DB writes in Rust" rule is preserved; this is a pure
reader). See ADR 0010.

## Schema source of truth

`migrate()` applies the desktop's migration files (`apps/desktop/migrations/*.sql`),
embedded by relative path via `include_str!`, in version order. It is a **bootstrap only**:
if the `templates` table already exists (a desktop-managed file, or a prior server run) it
does nothing. Schema evolution on a shared file is owned by the desktop's plugin-sql
migration chain, never this crate. When a new migration is added there, append it to the
`MIGRATIONS` array here in the same PR.

## Concurrency

`Connection` is `Send` but not `Sync`; it is wrapped in a `Mutex` so `SqliteStore` is
`Send + Sync` and can live behind `Arc<dyn DataStore>` in the router state. A 5-second
`busy_timeout` absorbs brief contention with the plugin-sql writer.

## SQLite linkage

`rusqlite` uses the `bundled` feature so it links the same vendored SQLite that
`tauri-plugin-sql` (sqlx → libsqlite3-sys) already pulls in; cargo unifies them into one
SQLite in the desktop binary rather than two.
