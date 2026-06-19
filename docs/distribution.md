# Distribution & Release Secrets

This document covers how `shalgalt-omr` is packaged, signed, and published, plus the full
GitHub Actions secret map the `release.yml` workflow consumes. It is the operator's
reference for cutting a release.

> Locked decisions live in [`.claude/PRPs/plans/shalgalt-omr-master.plan.md`](../.claude/PRPs/plans/shalgalt-omr-master.plan.md)
> §6.8 / §6.9 (auto-update, distribution priority) and the P6 sub-issues in §15. The
> distribution channel is **GitHub Releases triggered by `v*` tags** via `tauri-action`.
> Auto-update is **opt-in and OFF by default** so offline schools never see prompts.

## v0 signing posture (current)

Production EV/OV certificates and an Apple Developer account are **not yet available**.
The release pipeline therefore ships with a degraded-but-functional posture:

| Platform | v0 posture                                                                 |
| -------- | -------------------------------------------------------------------------- |
| Windows  | **Self-signed** certificate (see [`scripts/gen-self-signed-cert.ps1`](../scripts/gen-self-signed-cert.ps1)). SmartScreen will warn "unknown publisher". |
| macOS    | Notarization **skipped** behind a flag. Unsigned/ad-hoc `.dmg`; users must right-click → Open. |
| Linux    | AppImage + `.deb`; no signing required.                                    |

When real certificates arrive, the same GitHub Actions secrets are simply replaced with
the production credentials — no workflow or code changes are needed.

## Bundling native runtime libraries

`shalgalt-omr` depends on two native dynamic libraries that must travel inside the bundle,
not just at build time:

- **pdfium** — `pdfium-render`'s runtime library (`pdfium.dll` / `libpdfium.so` /
  `libpdfium.dylib`).
- **OpenCV (Windows only)** — `opencv_world<NNN>.dll` (e.g. `opencv_world4100.dll` for
  4.10.0). On Linux/macOS OpenCV is a system/build-time dependency, so nothing is
  bundled there.

These are fetched into `apps/desktop/resources/` by the fetch-binaries scripts and then
referenced from `tauri.conf.json#bundle.resources` (owned by Track-Config).

| Script                                                       | OS              | Fetches                                  |
| ------------------------------------------------------------ | --------------- | ---------------------------------------- |
| [`scripts/fetch-binaries.sh`](../scripts/fetch-binaries.sh)  | Linux / macOS   | pdfium dynamic library                   |
| [`scripts/fetch-binaries.ps1`](../scripts/fetch-binaries.ps1)| Windows         | `pdfium.dll` + `opencv_world<NNN>.dll`   |

Both scripts are **idempotent** (skip when the target already exists; pass `--force` /
`-Force` to re-download) and resolve the repo root from their own location, so they can
run from any working directory.

```bash
# Linux / macOS
scripts/fetch-binaries.sh

# Windows (PowerShell)
pwsh scripts/fetch-binaries.ps1
```

The Windows OpenCV download mirrors `.github/workflows/ci.yml`: the
`opencv-<version>-windows.exe` artifact is a 7-Zip self-extractor that unpacks to
`C:\tools\opencv\`, and the runtime DLL lives at `build\x64\vc16\bin`. Keep the OpenCV
version in `fetch-binaries.ps1` in lockstep with the `OPENCV_VERSION` env var in
`ci.yml` and with [`docs/adr/0001-windows-opencv-strategy.md`](adr/0001-windows-opencv-strategy.md).

## Updater signing keypair (minisign)

`tauri-plugin-updater` verifies update artifacts with a minisign keypair. Generate it
once with the Tauri CLI:

```bash
pnpm tauri signer generate -w ~/.tauri/shalgalt-omr.key
```

This produces:

- a **private key** (the `.key` file, password-protected during generation), and
- a **public key** (printed to stdout / written next to the private key).

Distribution of the keypair:

- The **public key** goes into `tauri.conf.json` at `plugins.updater.pubkey`. This file
  is owned by **Track-Config** — see the handoff note below.
- The **private key** becomes the `TAURI_SIGNING_PRIVATE_KEY` GitHub secret, and the
  password set during generation becomes `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

> **Handoff — Track-Config:** add the minisign **public key** to
> `tauri.conf.json` under `plugins.updater.pubkey`. The matching private key and its
> password live in the `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
> GitHub secrets. The pubkey is safe to commit; never commit the private key.

Treat the private key as a long-lived secret: if it is rotated, every previously shipped
client that has the old pubkey baked in can no longer verify new updates, so rotation
requires a fresh full (non-update) install.

## Windows code-signing certificate

For v0, generate a self-signed certificate with
[`scripts/gen-self-signed-cert.ps1`](../scripts/gen-self-signed-cert.ps1):

```powershell
pwsh scripts/gen-self-signed-cert.ps1
```

The script exports a password-protected `.pfx` and prints the thumbprint. Base64-encode
the PFX into the `WINDOWS_CERTIFICATE` secret and store its password in
`WINDOWS_CERTIFICATE_PASSWORD`. When an EV/OV certificate is available, export it to PFX
and overwrite the same two secrets — no other changes required.

## GitHub Actions secret map

`release.yml` reads the following secrets. **Required** secrets must be present for a
release to succeed. **Optional** secrets gate a capability: when an optional secret (or a
required member of an optional group) is absent, the corresponding step must **skip
gracefully** rather than fail the workflow. This is what lets us ship Windows builds
today while macOS signing/notarization is still uncredentialed.

### Updater signing — required

| Secret                              | Required | Purpose                                                                 |
| ----------------------------------- | -------- | ----------------------------------------------------------------------- |
| `TAURI_SIGNING_PRIVATE_KEY`         | Yes      | minisign private key that signs update artifacts for `tauri-plugin-updater`. |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`| Yes      | Password for the minisign private key (set during `tauri signer generate`). |

These are required because the updater feed must always be signed, even in v0; the
client refuses unsigned updates. If updater publishing is ever fully disabled for a
release, these may be treated as optional, but the default posture treats them as
required.

### Windows code signing

| Secret                          | Required | Purpose                                                                  |
| ------------------------------- | -------- | ------------------------------------------------------------------------ |
| `WINDOWS_CERTIFICATE`           | Optional | Base64-encoded PFX (self-signed in v0, EV/OV later). Absent ⇒ unsigned Windows bundle. |
| `WINDOWS_CERTIFICATE_PASSWORD`  | Optional | Password for the PFX. Only needed if `WINDOWS_CERTIFICATE` is set and password-protected. |

When `WINDOWS_CERTIFICATE` is absent, the Windows build still produces NSIS/MSI bundles —
they are simply unsigned and trigger a SmartScreen warning.

### macOS code signing + notarization — all optional

The entire macOS signing/notarization group is optional. If **any** required member of
the group is missing, the macOS signing/notarization steps **skip gracefully** and an
unsigned (ad-hoc) `.dmg` is produced instead.

| Secret                    | Required | Purpose                                                          |
| ------------------------- | -------- | ---------------------------------------------------------------- |
| `APPLE_CERTIFICATE`       | Optional | Base64-encoded Apple "Developer ID Application" `.p12`.          |
| `APPLE_CERTIFICATE_PASSWORD` | Optional | Password for the `.p12`.                                      |
| `APPLE_SIGNING_IDENTITY`  | Optional | The signing identity string (e.g. `Developer ID Application: …`).|
| `APPLE_ID`                | Optional | Apple ID email used for notarization.                            |
| `APPLE_PASSWORD`          | Optional | App-specific password for notarization.                          |
| `APPLE_TEAM_ID`           | Optional | Apple Developer Team ID used for notarization.                   |

Notarization is gated behind a flag per master plan §6.9 and P6-03; it is skipped in v0.

### Summary — required vs optional

- **Required:** `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- **Optional (Windows signing):** `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`.
- **Optional (macOS group — all-or-nothing):** `APPLE_CERTIFICATE`,
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`,
  `APPLE_TEAM_ID`.

> **Graceful skip is mandatory.** Absence of an optional secret must never fail the
> release. The corresponding signing/notarization step must detect the empty secret and
> skip, leaving an unsigned bundle. This keeps releases unblocked while certificates are
> being procured.

## Setting secrets

```bash
# Updater (required)
gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/shalgalt-omr.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD

# Windows (optional)
gh secret set WINDOWS_CERTIFICATE < cert.b64
gh secret set WINDOWS_CERTIFICATE_PASSWORD

# macOS (optional, all-or-nothing)
gh secret set APPLE_CERTIFICATE < apple-cert.b64
gh secret set APPLE_CERTIFICATE_PASSWORD
gh secret set APPLE_SIGNING_IDENTITY
gh secret set APPLE_ID
gh secret set APPLE_PASSWORD
gh secret set APPLE_TEAM_ID
```

No secrets are ever committed to the repository. The PFX, the base64 files, and the
minisign private key must stay out of version control.
