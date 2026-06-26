---
title: ADR 0001 — Windows OpenCV strategy
description: Provision OpenCV at Windows build time via the prebuilt upstream self-extractor rather than vcpkg, Chocolatey, or LFS-vendored binaries.
---

<Callout type="success" title="Accepted · 2026-05-02">
  **Deciders:** filename24 · **Supersedes:** README.md "Windows" placeholder
</Callout>

## Context

`opencv-rust` requires the OpenCV C++ libraries to be present at build time. Linux
(`libopencv-dev`) and macOS (`brew install opencv`) have one-line package-manager
solutions. Windows does not — there is no first-party package manager, and Tauri 2 +
opencv-rust + bindgen on Windows requires deliberate setup before CI or local builds
will succeed.

P0 scaffolding deferred this decision (README marked the section as "ADR pending"). With
CI dev-build matrix landing across Linux and macOS, Windows is the missing third leg.

## Options considered

| Option | Cold CI time | Cached CI | Local dev | DLL size | Module selectivity |
| ------ | -----------: | --------: | --------: | -------: | ------------------ |
| A. vcpkg manifest          | 30–60 min | 2–5 min | 30–60 min | 6–15 MB | Yes |
| **B. Prebuilt opencv.exe** | **5–10 min** | **2–3 min** | **5–10 min** | ~70 MB (`opencv_world`) | No |
| C. Chocolatey (`choco install opencv`) | 2–5 min | 2–5 min | 2–5 min | ~70 MB | No |
| D. LFS-vendored binaries   | <2 min | <2 min | LFS pull | Our choice | If we pre-build |

(See conversation log dated 2026-05-02 for detailed pros/cons of each option.)

## Decision

<Callout type="info" title="Decision">
  Adopt **Option B — prebuilt `opencv-X.Y.Z-windows.exe` from upstream releases**.
</Callout>

Rationale, in priority order:

1. **CI economy**. Windows GitHub Actions runners cost 2× Linux. A 5–10 min build keeps
   per-commit Windows CI affordable; vcpkg's 30–60 min cold builds would dominate the
   pipeline budget and discourage developers from triggering manual builds.
2. **Local dev parity**. The same five-minute install procedure works on a fresh dev
   machine. Anything more complicated (vcpkg, custom toolchain) raises the on-ramp for
   contributors.
3. **OpenCV usage is module-light**. shalgalt-omr only calls `core`, `imgproc`,
   `imgcodecs`, and `calib3d`. The 70 MB cost of `opencv_world` is real but bounded —
   it is the difference between a 80 MB and a 20 MB installer, both of which are
   acceptable for USB / school-network distribution at P0–P4. We are not optimising
   for App Store size limits.
4. **Reversibility**. A → B is hard (you would have to commit to vcpkg manifests, NuGet
   binary cache, and a different CI shape). B → A is easy: you replace the install
   step, set the same three env vars to vcpkg paths, drop `opencv_world` for individual
   `opencv_core` / `opencv_imgproc` libs. We can always migrate later when DLL diet
   becomes a real constraint.

### Pinned version

OpenCV **4.10.0**. Bumping requires updating both:

- [`.github/workflows/ci.yml`](https://github.com/filename24/shalgalt-omr/blob/stable/.github/workflows/ci.yml) — the `OPENCV_VERSION`
  job-level env var (drives the download URL and the `opencv_world<NNN>` DLL name).
- [`README.md`](https://github.com/filename24/shalgalt-omr/blob/stable/README.md) — the Windows install snippet.

The shortened DLL name follows OpenCV's convention: dots stripped, no padding. So
`4.10.0` → `opencv_world4100.dll`, `4.5.5` → `opencv_world455.dll`.

### Required env vars

opencv-rust auto-discovers OpenCV on Windows when these three are set:

| Variable | Value |
| -------- | ----- |
| `OPENCV_LINK_LIBS` | `opencv_world<NNN>` (e.g. `opencv_world4100`) |
| `OPENCV_LINK_PATHS` | `C:\tools\opencv\build\x64\vc16\lib` |
| `OPENCV_INCLUDE_PATHS` | `C:\tools\opencv\build\include` |

Plus `LIBCLANG_PATH=C:\Program Files\LLVM\bin` for bindgen, and the `bin` directory must
be on `PATH` so the runtime DLL is resolvable when launching the produced `.exe`.

`vc16` corresponds to the MSVC v142 toolchain (Visual Studio 2019). It is
forward-compatible with VS 2022's MSVC v143 — windows-latest GitHub runners use VS 2022
but link successfully against the vc16 OpenCV libs.

## Consequences

### Positive

- Windows CI matrix activates with ~5 min install + ~15 min build.
- Local Windows dev follows the same procedure as CI — copy/paste the env vars,
  download once, build.
- No vcpkg learning curve for contributors.

### Negative

- Installer size carries the full 70 MB `opencv_world<NNN>.dll` even though we use ~4
  modules. P5 packaging may revisit.
- Bound to opencv.org's GitHub Releases URL format. If upstream stops shipping
  `opencv-X.Y.Z-windows.exe`, we migrate to A or D.
- `opencv_world` lacks debug symbols. Diagnosing native crashes inside OpenCV on
  Windows is harder than on Linux's `libopencv-dev` (which has separate `-dbg`
  packages). Acceptable trade-off for now.

### Follow-up tickets

- **P5 — installer diet**. Re-evaluate vcpkg with feature-selective build if Windows
  installer footprint becomes a complaint vector. Target: < 30 MB total.
- **pdfium bundling** is orthogonal and tracked separately (see README "pdfium").
