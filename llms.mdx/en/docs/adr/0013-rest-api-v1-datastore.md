# ADR 0013 — REST `/v1` & DataStore seam (https://filename24.github.io/shalgalt-omr/en/docs/adr/0013-rest-api-v1-datastore)



<Callout type="success" title="Accepted · 2026-06-17">
  **Deciders:** filename24 · **Related:** P5-03 (read endpoints), P5-04 (write endpoints),
  P5-06 (OpenAPI) · **Related ADRs:** [0003](/adr/0003-cargo-workspace-and-crate-boundaries)
  (crate boundaries), [0014](/adr/0014-server-binary-and-rusqlite-store) (the hosts that
  inject a store).
</Callout>

## Context [#context]

Master plan §6.6 and §8.2 call for a versioned HTTP API exposing exams, templates, and
graded results, reachable identically from inside the Tauri app (`127.0.0.1`, no auth) and
from a standalone server (`0.0.0.0`, bearer auth). The router already lives in
`shalgalt-core` (Rule 4) but, before P5, served only `/healthz`.

Two constraints shape the design:

1. **Core is database-free.** `crates/shalgalt-core/CLAUDE.md` forbids any DB on the core
   dependency path; §8.2 pins core's dependencies to `serde`, `thiserror`, `anyhow`,
   `axum`, `tower-http`. The router must reach persistence without core knowing what the
   persistence *is*.
2. **Two hosts, one contract.** The desktop and the server must return byte-identical
   responses for the same data (master plan §10 P5 acceptance), so the endpoint logic
   cannot be duplicated per host.

## Decision [#decision]

<Callout type="info" title="Decision">
  Version all resource routes under `/v1/`, route persistence through a database-free
  `DataStore` trait in core, keep wire DTOs separate from `domain::*`, generate OpenAPI from
  `utoipa` annotations, and reuse the IPC error envelope for REST responses.
</Callout>

**1. Version under `/v1/`.** All resource routes are `/v1/{exams,templates,results}`.
`/healthz` and `/openapi.json` sit outside the version prefix (they are service metadata,
not part of the data contract). A breaking change to any `/v1/` shape requires a `/v2/`
and a new ADR.

**2. A `DataStore` trait is the persistence seam.** Core defines `DataStore` (object-safe,
`Send + Sync`) plus the DTOs that cross it (`ExamDto`, `TemplateDto`, `ResultDto`, the
`New*` request bodies). `AppState` holds an `Arc<dyn DataStore>`; every handler reads/writes
through it. Core ships an in-memory `MemoryStore` for tests. Each host injects a concrete
implementation (see ADR 0014). Core still touches no database.

**3. DTOs are separate from `domain::*`.** The wire shapes are their own types so the REST
contract can evolve without dragging the grading/IPC models. Opaque payloads
(`templates.json_schema`, `results.detail_answers`) are passed through as strings — the API
never reshapes the Rule 3 template format.

**4. OpenAPI via `utoipa`, served at `/openapi.json`.** Handlers carry `#[utoipa::path]` and
DTOs derive `ToSchema`, so the spec is generated from the same annotations that wire the
routes and cannot drift. `utoipa` is added to core as the single OpenAPI dependency
(sanctioned by §8.2's `api::router` surface). No Swagger UI is bundled — this is an
offline-first product; consumers point their own tooling at the JSON.

**5. Errors reuse the IPC envelope.** `AppError` gains an axum `IntoResponse` that emits the
existing `{ code, message }` body with a derived status (`BadRequest` → 400, `NotFound` →
404, else 500), so REST clients and the Tauri frontend read identical error shapes.

## Consequences [#consequences]

* The router is testable in-process over `MemoryStore` (no socket, no database) — see the
  `api` tests in core.
* Adding an endpoint means: a `DataStore` method, a handler with `#[utoipa::path]`, a route
  line, and a `paths(...)` entry. The OpenAPI document updates itself.
* `read_only()` is a shared `BadRequest` the desktop's read-only store returns for `create_*`
  — write endpoints exist in the contract but are rejected in local mode (see ADR 0014).
* `?exam_id=` on `/v1/results` filters the new `results.exam_id` column (migration 0005),
  so the spec's filter is a real column predicate rather than an approximation.

## Follow-up [#follow-up]

* Pagination on `/v1/results` is deferred until result volumes justify it; today the desktop
  browser paginates client-side.
* A future `/v2/` (if ever needed) keeps `/v1/` alive for one release for integrators.

<Cards>
  <Card href="/adr/0003-cargo-workspace-and-crate-boundaries" title="ADR 0003 — Cargo workspace & crate boundaries">
    The crate boundaries that keep the router in core and persistence out of it.
  </Card>
  <Card href="/adr/0014-server-binary-and-rusqlite-store" title="ADR 0014 — Standalone server & rusqlite store">
    The hosts that inject a concrete `DataStore` behind this seam.
  </Card>
</Cards>
