---
title: ADR 0017 — GitHub Releases distribution
description: Ship desktop bundles and the standalone server through GitHub Releases via tauri-action on v* tags.
---

<Callout type="success" title="Accepted · 2026-06-19">
  **Deciders:** filename24 · **Related issues:** P6-01 (`release.yml`), P6-02 (Windows bundle),
  P6-03 (standalone server asset) · **Related ADRs:** ADR 0001 (Windows OpenCV linkage — the
  runtime DLL that must ship alongside the installer), ADR 0014 (the `shalgalt-server` binary
  that becomes a Release asset), ADR 0018 (the updater that consumes these Release assets)
</Callout>

<Callout type="warn" title="Master-plan note">
  §16 reserves the nominal numbers 0011/0012 for the two P6 ADRs. Those file numbers were
  already taken by the P4 file-format / `age` ADRs, and the monotonic-no-gap naming rule
  (`README.md`) makes 0017/0018 the next free pair. §16's table must be updated in the same PR
  to point at the real numbers (0017 distribution, 0018 updater).
</Callout>

## Context

P6 turns the workspace into shippable artifacts. Master plan §6.8/§6.9 lock the channel
(GitHub Releases triggered by `v*` tags) and the OS priority (Windows 1, macOS 2,
Linux 3), but leave the *how* to an ADR: which workflow drives the build, which assets
land on a Release, and what we do without real code-signing certificates.

Three constraints make this non-trivial:

1. **No signing certificates are available yet.** A real Windows EV/OV certificate and an
   Apple Developer account are both pending. We cannot block the first tagged build on
   procurement.
2. **Native runtime libraries do not statically link.** Per ADR 0001 the Windows build
   links `opencv_world<NNN>.dll`, and the CV pipeline also depends on a `pdfium` shared
   library. Both must travel *with* the installer or the produced binary will not launch
   on a clean machine.
3. **Two products ship from one tag.** The desktop bundle (NSIS + MSI) and the headless
   `shalgalt-server` binary (ADR 0014) are both v1.0 deliverables and should be published
   from the same Release so versions never drift.

## Options considered

| Question | Option | Verdict |
| --- | --- | --- |
| Build driver | Hand-rolled `cargo tauri build` matrix in `release.yml` | **Rejected** — re-implements bundling, signing hooks, and Release upload that `tauri-action` already does. |
| | `tauri-apps/tauri-action` | **Chosen** — official action; creates/updates the Release, builds per-OS bundles, uploads assets, and emits the `latest.json` updater manifest when configured. |
| Trigger | Push to `stable` | **Rejected** — couples every release-branch merge to a publish; no clean version anchor. |
| | `v*` tag push | **Chosen** — matches master plan §6.8; the tag is the single version source of truth, and a Release is an explicit, intentional act. |
| Windows signing (now) | Block release until an EV/OV cert exists | **Rejected** — stalls the entire P6 milestone on a procurement timeline we do not control. |
| | Ship a **self-signed `v0`** build, swap in the real cert later via secrets | **Chosen** — unblocks distribution; SmartScreen warns but the artifact is installable and verifiable. |
| macOS signing/notarization (now) | Require Apple notarization | **Rejected** — needs a paid Apple Developer account we do not yet have. |
| | Notarization **optional, gated behind a flag**; skipped when no Apple secrets are present | **Chosen** — produces an (unnotarized) `.dmg` for testing; flips to notarized once credentials land, no workflow rewrite. |
| Server binary | Separate workflow / separate Release | **Rejected** — version drift between desktop and server. |
| | Same Release, extra build step uploads `shalgalt-server` as an asset | **Chosen** — one tag, one Release, both products in lockstep. |

## Decision

<Callout type="info" title="Decision">
  `release.yml` triggers on `v*` tags and drives `tauri-action`, publishing the desktop
  bundle and the standalone `shalgalt-server` binary to **one** Release per tag. Signing is
  staged (self-signed Windows now, optional macOS notarization), and native runtime libraries
  (`opencv_world`, `pdfium`) are bundled, not assumed.
</Callout>

**1. `release.yml` triggers on `v*` tags and drives `tauri-action`.** The workflow checks
out the tag, installs the per-OS native toolchain (the OpenCV setup from ADR 0001 on
Windows, system OpenCV on Linux/macOS), runs `pnpm build` for the SvelteKit assets, then
hands off to `tauri-action`, which builds the bundle, attaches it to the Release, and —
when the updater is configured (ADR 0018) — writes the signed `latest.json` manifest.

**2. OS rollout follows master-plan priority.** Windows (NSIS + MSI) is the first and
mandatory target. macOS (`.dmg`) and Linux (AppImage + `.deb`) jobs exist in the same
workflow but may be left cert-blocked / `continue-on-error` until P9 per master plan
§6.9; the *workflow shape* is built once so enabling an OS later is a flag flip, not a
rewrite.

**3. Signing posture is staged.**
- **Windows**: a **self-signed certificate** is used for the initial `v0`-class builds.
  The signing cert/password are read from repository secrets
  (`WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`). Swapping to a real EV/OV cert
  is a secret-value change only — the workflow does not change.
- **macOS**: notarization is **optional and skipped** when the Apple secrets
  (`APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, signing identity) are absent. The build
  still produces a `.dmg`; it is simply not notarized until an Apple Developer account
  exists.

**4. The standalone server is a Release asset.** A dedicated build step compiles
`apps/server` (`shalgalt-server`) for the same targets and uploads the binary (archived
with its license/readme) to the *same* Release created by `tauri-action`, so a tag yields
both the desktop installer and the headless server in one place.

**5. Native runtime libraries are bundled, not assumed.** Following ADR 0001, the Windows
job places `opencv_world<NNN>.dll` (and the `pdfium` shared library) where the bundler
includes them as Tauri `resources`/`externalBin` so the installed app and the server
binary both find their native dependencies on a clean machine.

## Consequences

### Positive

- A single `v*` tag produces a complete, downloadable Release: Windows installer +
  `shalgalt-server`, with macOS/Linux ready to enable.
- Distribution is unblocked today despite missing certificates; the cert story is a
  secret swap, not a code change.
- Versions never drift: desktop and server publish from one tag, one Release.
- Aligns with §6.10 — no telemetry, no update phone-home in the build pipeline itself.

### Negative

- **Self-signed Windows builds trigger SmartScreen warnings.** Users must click through
  "More info → Run anyway" until a real reputation-bearing cert is installed. Documented
  in the user manual (P7).
- **Unnotarized macOS `.dmg` is Gatekeeper-blocked** by default; testers must
  right-click → Open or clear the quarantine attribute. Resolved when notarization is
  switched on.
- **Installer footprint carries the full native libs** (`opencv_world` ~70 MB per
  ADR 0001, plus `pdfium`). Acceptable for USB / school-network distribution; an installer
  diet is the ADR 0001 follow-up, not a P6 blocker.
- **Coupled to upstream action + URL formats.** `tauri-action` and the OpenCV/pdfium
  download URLs are external dependencies; if they break, the workflow needs maintenance.

## Follow-up

- **Real Windows EV/OV certificate**: procure, store as secrets, retire the self-signed
  path. SmartScreen reputation accrues afterwards.
- **Apple Developer account + notarization**: enable the notarization flag and remove the
  skip once credentials exist.
- **Linux `.deb` + AppImage hardening**: confirm the bundled `pdfium`/OpenCV resolve on
  common distros before flipping the Linux job from optional to required (P9).
- **Installer diet** inherits ADR 0001's P5/P6 follow-up — feature-selective OpenCV build
  if footprint becomes a complaint vector.

<Cards>
  <Card href="/adr/0001-windows-opencv-strategy" title="ADR 0001 — Windows OpenCV strategy">
    The Windows OpenCV linkage; the runtime DLL this workflow must bundle with the installer.
  </Card>
  <Card href="/adr/0014-server-binary-and-rusqlite-store" title="ADR 0014 — Standalone server & rusqlite store">
    The `shalgalt-server` binary that becomes a Release asset alongside the desktop bundle.
  </Card>
  <Card href="/adr/0018-optin-updater" title="ADR 0018 — Opt-in updater">
    The updater that consumes these Release assets and the `latest.json` manifest.
  </Card>
</Cards>
