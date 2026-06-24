# ADR 0015 — API port & fallback (https://filename24.github.io/shalgalt-omr/en/docs/adr/0015-api-port-default-and-fallback)



<Callout type="success" title="Accepted · 2026-06-17">
  **Deciders:** filename24 · **Related issues:** P5-03/04/05 (the embedded + standalone HTTP
  API) · **Related ADRs:** ADR 0013 (the `/v1` surface), ADR 0014 (the two hosts that bind a
  port) · **Supersedes:** the `127.0.0.1:8080` / `0.0.0.0:8080` choice recorded in
  BLUEPRINT §2 and master plan §6.6.
</Callout>

## Context [#context]

P5 introduced two hosts that bind a TCP port: the desktop's embedded API (loopback) and the
standalone `shalgalt-server` (LAN). Both originally hard-coded port **8080**.

8080 is one of the most contended ports on a developer or teacher machine — Tomcat,
Jenkins, countless dev servers, and corporate proxies all default to it. Worse, the desktop
binder had two failure problems when 8080 was taken:

1. **Not configurable.** The port was a hard-coded literal; a teacher had no way to move it.
2. **A bind failure bricked the whole app, silently.** `api::spawn(...).await?` propagated
   the error out of `bootstrap`, which &#x2A;skipped the subsequent `app.manage(state)`*. The
   window still opened, but every IPC command that needs `State<AppState>` (scanning, PDF
   generation, xlsx export) then failed — with nothing but one line in the log file. A
   single port collision turned the app into a silent brick.

The standalone server failed more gracefully (it exits with a clear error and `--bind` is
already configurable), but shared the contended default.

## Options considered [#options-considered]

| Question                                                 | Option                               | Verdict                                                                                                                                                                   |
| -------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Keep 8080?                                               | Yes, document the collision risk     | **Rejected** — the risk is real on the exact machines we target (school PCs) and the failure mode is catastrophic.                                                        |
| New default port                                         | A common alt (8000/8888/9000)        | **Rejected** — equally contended.                                                                                                                                         |
|                                                          | A registered-range, uncommon port    | **Chosen — 22345.** In the IANA *registered* range (1024–49151) so the OS never hands it out as an ephemeral port, IANA-unassigned, and nowhere near common dev defaults. |
|                                                          | An OS-assigned ephemeral port (`:0`) | **Rejected as the default** — integrators want a stable, documentable port; pure ephemeral makes discovery mandatory every run.                                           |
| Desktop behavior when the port is busy                   | Hard fail                            | **Rejected** — the embedded API is an *optional* integration surface; the app must keep working without it.                                                               |
|                                                          | Auto-fallback to the next free port  | **Chosen** — try `[desired, desired+16)`, bind the first free one, and *isolate the failure* so `app.manage(state)` always runs.                                          |
| Server behavior when the port is busy                    | Auto-fallback                        | **Rejected** — a LAN server advertises a fixed port to clients; silently moving it would strand them. Hard fail so the operator picks another `--bind`.                   |
| How integrators discover the bound port (after fallback) | Endpoint file + IPC command          | **Chosen** — the desktop writes `app_data_dir/api-endpoint.json` (`{ port, base_url }`) and exposes an `api_info` command.                                                |
| Developer port override                                  | New CLI flag                         | **Rejected for desktop** — it has no CLI; the server already has `--bind`.                                                                                                |
|                                                          | Environment variable                 | **Chosen — `SHALGALT_API_PORT`** for the desktop. It is a developer knob (not user-facing UI), so it does not touch the Mongolian-only UI rule.                           |

## Decision [#decision]

<Callout type="info" title="Decision">
  Default both hosts to port **22345**. The desktop **auto-falls-back** to the next free port
  (`[desired, desired+16)`) and isolates a bind failure so IPC always survives; the server
  hard-fails so a fixed advertised port never silently moves. `SHALGALT_API_PORT` is the
  developer override.
</Callout>

**1. Default port → 22345**, for both hosts. Desktop binds `127.0.0.1:22345`; the server's
`--bind` default becomes `0.0.0.0:22345`.

**2. Desktop auto-fallback.** `api::spawn` resolves a *desired* port (from
`SHALGALT_API_PORT`, else 22345) and binds the first free port in `[desired, desired+16)`.
Only `AddrInUse` triggers fallback; other bind errors return immediately so a real
misconfiguration is not masked. The chosen port rides along in `ApiHandle::port()`.

**3. Failure isolation (the bug fix).** `bootstrap` now `match`es on `api::spawn`: on
success it manages the handle and an `ApiInfo::running(port)`; on failure it logs and
manages `ApiInfo::disabled()&#x60;. **`app.manage(state)` always runs**, so a missing API never
breaks IPC.

**4. Discovery.** On a successful bind the desktop writes `api-endpoint.json`
(`{ "port", "base_url" }`) into the app data dir and exposes an `api_info` IPC command
returning `{ running, port, base_url }`. Integrators read either instead of assuming 8080.

**5. Developer override.** `SHALGALT_API_PORT` pins the desktop's starting port (fallback
still applies). The server keeps `--bind` for explicit control and does **not** fall back.

## Consequences [#consequences]

* A teacher whose machine already runs something on 22345 gets a working app on 22346+,
  with the real port discoverable from `api-endpoint.json`. No silent brick.
* External integrations must stop hard-coding 8080. The endpoint file + `api_info` command
  are the supported discovery paths.
* The server's behavior is intentionally stricter than the desktop's: identical default
  port, but no fallback. This asymmetry is documented in both `AGENTS.md` files.
* BLUEPRINT §2 and master plan §6.6 are updated in the same PR (both now point here).

## Follow-up [#follow-up]

* When a developer-settings surface lands, show `api_info` (port + base URL) there so users
  can copy the integration endpoint without opening the data dir.
* Consider a server-side `--port-fallback` opt-in if headless deployments ever want it; not
  needed today.

<Cards>
  <Card href="/adr/0013-rest-api-v1-datastore" title="ADR 0013 — REST `/v1` & DataStore seam">
    The `/v1` surface served on this port.
  </Card>
  <Card href="/adr/0014-server-binary-and-rusqlite-store" title="ADR 0014 — Standalone server & rusqlite store">
    The two hosts (desktop, standalone server) that bind the port.
  </Card>
</Cards>
