# Architecture Decisions (https://filename24.github.io/shalgalt-omr/mn/docs/adr)



Architecture Decision Records (ADRs) capture the choices that shaped shalgalt-omr: the
context, the options weighed, the decision, and the consequences we accepted. They exist so
a future contributor can understand *why* the system looks the way it does without
re-litigating settled questions.

<Callout type="info" title="When to write one">
  Write an ADR when a decision is **hard to reverse** (build toolchain, file format, IPC
  contract), when **multiple credible options** exist with non-obvious trade-offs, or when a
  future contributor would otherwise ask the same question again. Small, local, easily
  reversible changes do not need one — just write the code.
</Callout>

<Callout type="warn" title="Locked decisions">
  Every ADR below is **Accepted**. Re-opening a locked decision requires a *new* ADR plus a
  master-plan update in the same PR — see the [master plan](https://github.com/filename24/shalgalt-omr/blob/stable/.claude/PRPs/plans/shalgalt-omr-master.plan.md)
  and the [contribution guide](/dev/contributing).
</Callout>

## Foundation & build [#foundation--build]

<Cards>
  <Card href="/adr/0001-windows-opencv-strategy" title="ADR 0001 — Windows OpenCV strategy">
    How OpenCV is provisioned at build time across Windows, macOS, and Linux.
  </Card>

  <Card href="/adr/0003-cargo-workspace-and-crate-boundaries" title="ADR 0003 — Cargo workspace & crate boundaries">
    The monorepo split into `apps/*` hosts and pure `crates/*` libraries.
  </Card>

  <Card href="/adr/0004-ts-rs-type-codegen" title="ADR 0004 — TypeScript bindings via ts-rs">
    Generating the frontend's domain types from Rust so they never drift.
  </Card>
</Cards>

## PDF generation & templates [#pdf-generation--templates]

<Cards>
  <Card href="/adr/0002-pdf-generator-printpdf" title="ADR 0002 — PDF generator: printpdf">
    Choosing a pure-Rust PDF engine with embedded Noto fonts over native bindings.
  </Card>

  <Card href="/adr/0005-pdf-ipc-contract" title="ADR 0005 — PDF generation IPC contract">
    Path-string commands and on-disk results — never bytes across the IPC boundary.
  </Card>

  <Card href="/adr/0007-canvas-wrapper-and-layout-modules" title="ADR 0007 — Canvas wrapper & layout modules">
    A drawing-surface abstraction with per-element layout modules in `shalgalt-pdf`.
  </Card>

  <Card href="/adr/0008-bubble-label-position" title="ADR 0008 — Bubble label position">
    Why option labels are printed inside the bubble circle.
  </Card>
</Cards>

## UI & theming [#ui--theming]

<Cards>
  <Card href="/adr/0006-light-theme-and-comfort-typography" title="ADR 0006 — Light theme & comfort typography">
    Light-by-default theming plus an opt-in comfortable type scale.
  </Card>
</Cards>

## Computer-vision pipeline [#computer-vision-pipeline]

<Cards>
  <Card href="/adr/0009-aruco-markers" title="ADR 0009 — ArUco corner markers">
    `DICT_6X6_50` markers (IDs 0–3, TL/TR/BR/BL) replace anonymous corner squares.
  </Card>

  <Card href="/adr/0010-confidence-band" title="ADR 0010 — Fill measurement & confidence">
    Per-bubble fill scoring, decision bands, and the `needs_review` rule.
  </Card>

  <Card href="/adr/0016-partial-marker-homography" title="ADR 0016 — Partial-marker homography">
    Recovering perspective when fewer than four markers are detected.
  </Card>
</Cards>

## Project file format [#project-file-format]

<Cards>
  <Card href="/adr/0011-shalgalt-file-format" title="ADR 0011 — `.shalgalt` project file">
    A zip container with an always-plaintext manifest.
  </Card>

  <Card href="/adr/0012-age-encryption" title="ADR 0012 — Optional age encryption">
    Passphrase-based, ASCII-armored `age` encryption for project payloads.
  </Card>
</Cards>

## HTTP API [#http-api]

<Cards>
  <Card href="/adr/0013-rest-api-v1-datastore" title="ADR 0013 — REST `/v1` & DataStore seam">
    Versioned routes, a storage seam in core, and OpenAPI via `utoipa`.
  </Card>

  <Card href="/adr/0014-server-binary-and-rusqlite-store" title="ADR 0014 — Standalone server & rusqlite store">
    A shared `rusqlite` `DataStore`, splitting the read path from plugin-sql writes.
  </Card>

  <Card href="/adr/0015-api-port-default-and-fallback" title="ADR 0015 — API port & fallback">
    The uncommon default port (22345), desktop auto-fallback, and env override.
  </Card>
</Cards>

## Distribution & documentation [#distribution--documentation]

<Cards>
  <Card href="/adr/0017-github-releases-distribution" title="ADR 0017 — GitHub Releases distribution">
    Shipping signed bundles through GitHub Releases with `tauri-action`.
  </Card>

  <Card href="/adr/0018-optin-updater" title="ADR 0018 — Opt-in updater">
    `tauri-plugin-updater` with a static `gh-pages` manifest, default off.
  </Card>

  <Card href="/adr/0019-docs-fumadocs-and-gh-pages" title="ADR 0019 — Documentation site">
    This Fumadocs site, deployed to GitHub Pages.
  </Card>
</Cards>
