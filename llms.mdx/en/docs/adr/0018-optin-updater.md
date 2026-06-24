# ADR 0018 — Opt-in updater (https://filename24.github.io/shalgalt-omr/en/docs/adr/0018-optin-updater)



<Callout type="success" title="Accepted · 2026-06-19">
  **Deciders:** filename24 · **Related issues:** P6-04 (integrate `tauri-plugin-updater`) ·
  **Related ADRs:** ADR 0017 (the Release assets and signatures this manifest points at),
  ADR 0006 (the preferences surface where the opt-in toggle lives next to the theme controls)
</Callout>

<Callout type="warn" title="Master-plan note">
  §16 nominally reserves number 0012 for "Opt-in updater"; that file number is taken by the P4
  `age`-encryption ADR. Under the monotonic-no-gap naming rule this ADR is **0018** (paired
  with 0017 for distribution). §16's table must be reconciled to the real numbers in the same PR.
</Callout>

## Context [#context]

Master plan §6.8 commits to `tauri-plugin-updater` for in-app updates, and §6.10 forbids
all telemetry, crash reporting, and analytics. The product's primary user is a teacher on
an **offline school network**. These two facts collide on a single question: an
auto-updater is, by nature, a process that reaches out to the network on launch.

The hard requirements:

* **Offline schools must never see an update prompt.** A spinner or "update available"
  dialog on a machine with no internet is a support burden and erodes trust in an
  offline-first tool.
* **No phone-home.** A version check is itself a network beacon (timestamp, IP, app
  version). §6.10 rules out anything that smells like telemetry, so a check must only
  happen when the user has explicitly asked for it.
* **Updates must still be verifiable.** When a connected teacher *does* opt in, the
  manifest and bundles must be signature-checked so a tampered update cannot install.

Where does the update manifest live, and what is the default behavior?

## Options considered [#options-considered]

| Question         | Option                                          | Verdict                                                                                                                                    |
| ---------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Default behavior | Auto-check on every launch (industry default)   | **Rejected** — violates §6.10 (silent phone-home) and shows prompts on offline machines.                                                   |
|                  | **Opt-in, default OFF**; teacher flips a toggle | **Chosen** — no network access until explicitly enabled; offline schools stay silent.                                                      |
| Manifest hosting | Dynamic update server                           | **Rejected** — a server to run/secure, and an always-on endpoint we'd have to keep alive; over-engineered for a static artifact.           |
|                  | Same Release asset                              | **Considered** — works, but the endpoint URL changes per tag, awkward to pin.                                                              |
|                  | **Static JSON on the `gh-pages` branch**        | **Chosen** — free hosting, a stable URL, no infrastructure; matches §6.8's "static JSON" model.                                            |
| Signature trust  | Trust TLS / GitHub only                         | **Rejected** — TLS authenticates the host, not the payload; we want payload integrity independent of transport.                            |
|                  | **`tauri-plugin-updater` minisign signatures**  | **Chosen** — the bundle + manifest are signed with the updater keypair; the public key is compiled into the app and verifies every update. |

## Decision [#decision]

<Callout type="info" title="Decision">
  Wire `tauri-plugin-updater` but keep it **opt-in, default OFF**: no check fires on launch
  until a teacher enables it. The update manifest is a static `latest.json` on `gh-pages`, and
  every update is **minisign-verified** against a public key embedded in the app.
</Callout>

**1. The updater is opt-in and OFF by default.** `tauri-plugin-updater&#x60; is wired in, but
no update check fires on launch. A **"Check for updates" toggle** lives in app
preferences, next to the theme / comfortable-typography controls (ADR 0006). The plugin
only contacts the network after a teacher turns it on (or presses an explicit
"Check now"). On a fresh install and on every offline machine, the app makes **zero**
update-related network calls — satisfying §6.10 and the offline-school requirement.

**2. The update manifest is a static `latest.json` on `gh-pages`.** The canonical endpoint
is:

```
https://filename24.github.io/shalgalt-omr/latest.json
```

It is published by the `release.yml` workflow (ADR 0017) when a `v*` tag is built, and
describes the latest version, per-platform bundle URLs (the Release assets), and their
signatures. No dynamic server exists; the endpoint is a flat file served by GitHub Pages.

**3. Updates are signature-verified.** Bundles and the manifest are signed with the
`tauri-plugin-updater` (minisign) keypair. The **public key is embedded in the app**
(`tauri.conf.json` updater config); the **private key lives only in CI secrets**
(`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`) and is never
committed. The plugin refuses any update whose signature does not verify against the
embedded public key, so transport compromise alone cannot push a malicious update.

**4. UI strings follow the language rule.** The toggle label, the "update available" copy,
and any error surface are **Mongolian**, referenced by key from the P1 string-table — not
hard-coded. The updater plugin's own `AppError`-style codes stay English (stable
identifiers, mapped to Mongolian in the table).

## Consequences [#consequences]

### Positive [#positive]

* **Offline schools never see a prompt.** Default-OFF means no network access until a
  teacher chooses it — the strongest possible answer to the offline-first requirement.
* **No telemetry / no phone-home** by default, satisfying §6.10. A version check only
  exists as a user-initiated action.
* **Zero update infrastructure.** A static `gh-pages` file is free, has a stable URL, and
  cannot "go down" the way a custom update server can.
* **Tamper-resistant.** Signature verification against an embedded public key protects the
  update path independently of TLS.
* **Teacher control.** Connected users who *want* updates get a clear, deliberate switch.

### Negative [#negative]

* **Most users stay on whatever they installed.** Because the default is OFF, the
  population skews toward old versions; security/bug fixes propagate only to opt-ins or via
  re-download. Acceptable given the offline-first, no-telemetry priorities — manual
  re-install from the Release page is always available.
* **Signing-key custody is critical.** Losing `TAURI_SIGNING_PRIVATE_KEY` means no machine
  can verify a new update (every client trusts only the embedded public key); rotating it
  requires shipping a new app version with the new public key first. Key backup/rotation is
  a documented operational follow-up.
* **`gh-pages` is a single point of distribution** for the manifest. If the Pages site is
  unavailable, opt-in checks fail gracefully (no update found) — they do not block the app.

## Follow-up [#follow-up]

* **Signing-key backup + rotation runbook**: document where the private key is stored, how
  it is backed up, and the version-bridged rotation procedure (P7 docs).
* **`latest.json` generation in `release.yml`**: confirm the manifest schema and per-OS
  asset URLs are emitted correctly once the macOS/Linux jobs (ADR 0017) are enabled.
* **String-table keys** for the updater UI land with the toggle implementation (P6-04).
* **Verify default-OFF** is asserted in the v1.0 acceptance checklist (master plan §17 —
  "Auto-updater is **off** by default").

<Cards>
  <Card href="/adr/0017-github-releases-distribution" title="ADR 0017 — GitHub Releases distribution">
    The Release assets and signatures the `latest.json` manifest points at.
  </Card>
  <Card href="/adr/0006-light-theme-and-comfort-typography" title="ADR 0006 — Light theme & comfort typography">
    The preferences surface where the opt-in toggle lives next to the theme controls.
  </Card>
</Cards>
