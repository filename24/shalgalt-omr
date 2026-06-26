# shalgalt-omr

> Local-First OMR Grading IDE — Tauri 2 + SvelteKit + Rust.
>
> Full spec: [`docs/BLUEPRINT.md`](docs/BLUEPRINT.md). Folder structure and module
> responsibilities: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Documentation

The full documentation site (developer docs in English + a Mongolian user manual + the HTTP
API reference) is built with **Fumadocs** and published to GitHub Pages:

- **Site**: https://filename24.github.io/shalgalt-omr/ (default Mongolian; English via the
  locale switcher).
- **Sources**: developer docs in [`docs/dev/`](docs/dev), user manual in
  [`docs/user/`](docs/user), and the Fumadocs app in [`docs/site/`](docs/site). See
  [ADR 0019](docs/adr/0019-docs-fumadocs-and-gh-pages.md) for the toolchain and deploy model.
- **Build locally**: `cd docs/site && pnpm install && pnpm build` (static export into
  `docs/site/out/`).

## Stack

| Layer       | Tech                                                                  |
| ----------- | --------------------------------------------------------------------- |
| Frontend    | SvelteKit 5 (adapter-static), TailwindCSS v4, svelte-konva, paneforge |
| UI kit      | shadcn-svelte (bits-ui + tailwind-variants), @lucide/svelte icons     |
| Native UI   | Tauri 2.0                                                             |
| Core        | Rust — opencv-rust, pdfium-render, rust_xlsxwriter                    |
| Persistence | tauri-plugin-sql + SQLite (single local file)                         |
| API         | axum (background, default port 22345 — ADR 0015)                      |

## System Prerequisites

This project uses native dependencies; the following system packages must be available
before building.

### Debian/Ubuntu

```bash
sudo apt update
sudo apt install -y \
  build-essential pkg-config libssl-dev \
  libopencv-dev clang libclang-dev \
  libgtk-3-dev libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

### macOS (Homebrew)

```bash
brew install opencv pkg-config llvm
```

### Windows

Per [ADR 0001](docs/adr/0001-windows-opencv-strategy.md) we use the upstream OpenCV
prebuilt self-extractor (option B). LLVM (for opencv-rust's bindgen) ships with recent
Visual Studio installs; install separately via [llvm.org](https://releases.llvm.org/) if
missing.

```powershell
# 1) Download and extract OpenCV 4.10.0 to C:\tools\opencv
$url = "https://github.com/opencv/opencv/releases/download/4.10.0/opencv-4.10.0-windows.exe"
Invoke-WebRequest $url -OutFile opencv.exe
Start-Process .\opencv.exe -ArgumentList '-o"C:\tools" -y' -Wait
Remove-Item opencv.exe

# 2) Set env vars (persist via setx, or add to your shell profile)
$env:OPENCV_LINK_LIBS     = "opencv_world4100"
$env:OPENCV_LINK_PATHS    = "C:\tools\opencv\build\x64\vc16\lib"
$env:OPENCV_INCLUDE_PATHS = "C:\tools\opencv\build\include"
$env:LIBCLANG_PATH        = "C:\Program Files\LLVM\bin"

# 3) Add the OpenCV runtime DLL directory to PATH so the produced exe can resolve it
$env:Path = "C:\tools\opencv\build\x64\vc16\bin;$env:Path"
```

> `scripts/fetch-binaries.ps1` performs step 1 for you — it extracts OpenCV 4.10.0 to
> `C:\tools\opencv` and copies the runtime `opencv_world4100.dll` into
> `apps/desktop/resources/`. You still set steps 2–3 (the build-time env vars and `PATH`)
> in your shell yourself, since the Rust compile and link read them at build time.

Bumping OpenCV requires updating the version in three places: the snippet above, the
`OPENCV_VERSION` job-level env in [`.github/workflows/ci.yml`](.github/workflows/ci.yml),
and the ADR itself.

### Native runtime libraries (pdfium + Windows OpenCV DLL)

`pdfium-render` loads the pdfium dynamic library at runtime, and on Windows the OpenCV
runtime DLL must ship next to the executable. Both land in `apps/desktop/resources/`
(bundled via `tauri.conf.json#bundle.resources`) through a single idempotent script:

| Platform      | Command                           | Fetches                               |
| ------------- | --------------------------------- | ------------------------------------- |
| Windows       | `pwsh scripts/fetch-binaries.ps1` | `pdfium.dll` + `opencv_world4100.dll` |
| Linux / macOS | `scripts/fetch-binaries.sh`       | `libpdfium.so` / `libpdfium.dylib`    |

You normally never run these by hand: they fire automatically on `pnpm install`
(postinstall) and again before `pnpm tauri dev` / `pnpm tauri build`, so a fresh clone
needs no manual step. They are idempotent — files already present are skipped; pass
`-Force` (ps1) / `--force` (sh) to refresh — and are skipped automatically in CI or when
`SHALGALT_SKIP_FETCH_BINARIES=1` is set. Run the platform command above manually if you
ever need to repopulate `resources/` (e.g. you cleaned it, or installed with
`--ignore-scripts`).

## Install & Dev

```bash
pnpm install
pnpm tauri dev
```

> `pnpm install` runs the native-binary fetch automatically (postinstall), and
> `pnpm tauri dev` / `pnpm tauri build` re-check it first, so `apps/desktop/resources/` is
> populated without any manual step. See the **Native runtime libraries** section above to
> run it by hand or to opt out with `SHALGALT_SKIP_FETCH_BINARIES=1`.

> On first run, the SQL plugin creates the SQLite file in the OS-specific
> `ProjectDirs::data_dir` (e.g. macOS `~/Library/Application Support/dev.filename.shalgalt-omr/`,
> Linux `~/.local/share/dev.filename.shalgalt-omr/`) and applies migrations automatically.

## Project Layout (summary)

The repo is a Cargo workspace as of P2-01 — Rust crates live under `apps/` and
`crates/`, the SvelteKit frontend stays at the root, and a single
`Cargo.lock` / `target/` lives at the workspace root.

```
docs/             — BLUEPRINT, ARCHITECTURE, ADRs
src/              — SvelteKit frontend (routes, lib/ipc, lib/db, lib/stores, lib/components, lib/types)
apps/
  desktop/        — Tauri 2 desktop app (was `src-tauri/`)
    migrations/   — SQL files embedded into Rust via `include_str!`
    src/
      domain/     — pure models (moves to crates/shalgalt-core in P2-02)
      scan/       — CV pipeline (moves to crates/shalgalt-cv in P2-03)
      grading/    — grading engine (moves to crates/shalgalt-core in P2-02)
      export/     — xlsx (moves to crates/shalgalt-core in P2-02)
      api/        — axum background server (Rule 4 — router moves to shalgalt-core in P2-02)
      commands/   — tauri::command IPC thin layer (Rule 1·2) — stays here
  server/         — standalone axum binary (implemented in P5-05; placeholder today)
crates/
  shalgalt-core/  — domain + grading + xlsx export + axum router (P2-02)
  shalgalt-pdf/   — printpdf-based OMR PDF generator (P2-05)
  shalgalt-cv/    — CV pipeline facade (P2-03)
  shalgalt-fileformat/ — `.shalgalt` zip + age encryption (P4-01)
```

Common commands run at the workspace root:

```bash
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
pnpm tauri dev              # spawns Tauri pointing at apps/desktop/tauri.conf.json
```

## License

MIT
