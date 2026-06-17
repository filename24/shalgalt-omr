# `apps/server` — Standalone HTTP Server (P5)

Headless `shalgalt-server` binary that exposes the same REST surface as the desktop app's
embedded axum server, intended for school-server deployments where grading happens on a
shared machine and clients connect over the LAN.

> Status: **implemented** (P5-05). `src/lib.rs` assembles the app; `src/main.rs` parses the
> CLI and serves. The split keeps the auth path integration-testable (`tests/auth.rs`).

> Repo-level rules, language conventions, and locked decisions live in
> [`/AGENTS.md`](../../AGENTS.md). This file describes only what is specific to this app.

## Role

`apps/server` reuses `shalgalt_core::api::router(state)` — the same `Router` consumed by
`apps/desktop` — over a read-write `shalgalt_store::SqliteStore`, and adds:

- bearer-token auth from `SHALGALT_API_TOKEN` (Server mode requirement, master plan §6.6)
- CORS allow-list from `--allow-origin` (never `Any`)
- bind address from `--bind` (default `0.0.0.0:8080`, vs. the desktop's `127.0.0.1`)
- structured `tracing` to stdout for systemd / Docker journals
- (Eventually) a small admin-token rotation endpoint behind the same auth

## CLI

```text
shalgalt-server --db PATH [--bind 0.0.0.0:8080] [--allow-origin ORIGIN]...
SHALGALT_API_TOKEN=…   # optional; when set, every request needs `Authorization: Bearer …`
```

`--db` is created and migrated (via `shalgalt-store`) if absent. When `SHALGALT_API_TOKEN`
is unset the API serves unauthenticated with a loud warning — fine on a trusted LAN, never
on an open network.

The crate has **no Tauri dependency** on the path. That guarantee is enforced by the
workspace dependency graph: pulling in `tauri` here would also pull it into
`shalgalt-core`, which we deliberately keep tauri-free.

## Critical Rules — server application

### Rule 4 — axum Server Independence

The desktop app spawns the same router on a tokio task and shuts it down via oneshot.
This binary owns the entire process, so the harness is simpler — `axum::serve` runs
to completion and shutdown is via SIGTERM / SIGINT only.

What does **not** change between the two apps:

- The router is built by `shalgalt_core::api::router(state)`. Do not fork it.
- The CORS layer comes from `shalgalt_core::api::cors::allow_list(&origins)`. Never `Any`.
- Versioned under `/v1/`. Breaking changes require a bump (`/v2/`) and an ADR.

The app is assembled by `build_app(state, allow_origins, token)` in `src/lib.rs`. Layer
order is deliberate: auth is applied **inside** CORS, so the CORS layer is outermost and
answers the preflight `OPTIONS` (which carries no `Authorization`) before the bearer guard
runs.

### Auth

Server mode is the only place where auth is enforced. The desktop binds to `127.0.0.1` and
skips bearer checks because the only client that can reach it is the local webview.

```text
Authorization: Bearer <SHALGALT_API_TOKEN>
```

Missing or mismatched token → `401 Unauthorized` (constant-time compare, in
`shalgalt_core::api::auth`). The token is read from the environment at startup; rotation is
documented but not yet automated.

## Layout

- `src/lib.rs` — `Cli` (clap), `build_app`, `serve`, `init_tracing`, `shutdown_signal`.
- `src/main.rs` — parses `Cli`, calls `serve`.
- `tests/auth.rs` — `oneshot` checks: missing/wrong token → 401, correct token → 200,
  no-token mode serves unauthenticated.
