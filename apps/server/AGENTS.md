# `apps/server` — Standalone HTTP Server (P5)

Headless `shalgalt-server` binary that exposes the same REST surface as the desktop app's
embedded axum server, intended for school-server deployments where grading happens on a
shared machine and clients connect over the LAN.

> Status: **placeholder**. The real binary lands in P5-05 per the master plan §10. The
> current `src/main.rs` is a `println!` stub that keeps the workspace member resolvable
> for `cargo check`.

> Repo-level rules, language conventions, and locked decisions live in
> [`/AGENTS.md`](../../AGENTS.md). This file describes only what is specific to this app.

## Role

`apps/server` reuses `shalgalt_core::api::routes` — the same `Router` builder consumed by
`apps/desktop` — and adds:

- bearer-token auth from `SHALGALT_API_TOKEN` (Server mode requirement, master plan §6.6)
- CORS allow-list (configured via env var, never `Any`)
- bind address: `0.0.0.0` (vs. the desktop's `127.0.0.1`)
- structured `tracing` to stdout for systemd / Docker journals
- (Eventually) a small admin-token rotation endpoint behind the same auth

The crate has **no Tauri dependency** on the path. That guarantee is enforced by the
workspace dependency graph: pulling in `tauri` here would also pull it into
`shalgalt-core`, which we deliberately keep tauri-free.

## Critical Rules — server application

### Rule 4 — axum Server Independence

The desktop app spawns the same router on a tokio task and shuts it down via oneshot.
This binary owns the entire process, so the harness is simpler — `axum::serve` runs
to completion and shutdown is via SIGTERM / SIGINT only.

What does **not** change between the two apps:

- The router is built by `shalgalt_core::api::routes::build()`. Do not fork it.
- The CORS layer comes from `shalgalt_core::api::cors`. Server mode passes an explicit
  allow-list `Vec<String>` (read from env). Never `Any`.
- Versioned under `/v1/`. Breaking changes require a bump (`/v2/`) and an ADR.

### Auth (P5)

Server mode is the only place where auth is enforced. The desktop app binds to
`127.0.0.1` and skips bearer checks because the only client that can reach it is the
local webview.

```text
Authorization: Bearer <SHALGALT_API_TOKEN>
```

Missing or mismatched token → `401 Unauthorized`. The token comes from the environment
on startup; rotation is documented but not yet automated.

## When implementing P5-05

The implementation order that keeps `cargo check` green at every step:

1. Add `tokio` (`rt-multi-thread`, `signal`, `macros`) and `axum`, `tower`, `tower-http`
   to `Cargo.toml`.
2. Wire `tracing-subscriber` (`fmt` + `EnvFilter`) — match `apps/desktop`'s subscriber
   shape.
3. Read `SHALGALT_API_TOKEN`, `SHALGALT_BIND_ADDR` (default `0.0.0.0:8080`),
   `SHALGALT_CORS_ALLOW_ORIGINS` (comma-separated). Fail fast if any required value is
   missing.
4. Build the router with `shalgalt_core::api::routes::build()` and layer the auth
   middleware + CORS allow-list on top.
5. `axum::serve(...).with_graceful_shutdown(shutdown_signal()).await`.
6. Add an integration test under `tests/` that boots the server on an ephemeral port,
   issues a request without `Authorization`, and asserts `401`.
