# shalgalt-omr

> Local-First OMR Grading IDE — Tauri 2 + SvelteKit + Rust.
>
> Full spec: [`docs/BLUEPRINT.md`](docs/BLUEPRINT.md). Folder structure and module
> responsibilities: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Stack

| Layer       | Tech                                                                  |
| ----------- | --------------------------------------------------------------------- |
| Frontend    | SvelteKit 5 (adapter-static), TailwindCSS v4, svelte-konva, paneforge |
| UI kit      | shadcn-svelte (bits-ui + tailwind-variants), @lucide/svelte icons     |
| Native UI   | Tauri 2.0                                                             |
| Core        | Rust — opencv-rust, pdfium-render, rust_xlsxwriter                    |
| Persistence | tauri-plugin-sql + SQLite (single local file)                         |
| API         | axum (background, port 8080)                                          |

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

Bumping OpenCV requires updating the version in three places: the snippet above, the
`OPENCV_VERSION` job-level env in [`.github/workflows/ci.yml`](.github/workflows/ci.yml),
and the ADR itself.

### pdfium

`pdfium-render` looks up the pdfium dynamic library at runtime. Download a release for
your OS from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) and
place it next to the executable, or extend `LD_LIBRARY_PATH` (Linux) / `DYLD_LIBRARY_PATH`
(macOS) / `PATH` (Windows). P4 will add a script that bundles it into
`tauri.conf.json#bundle.resources`.

## Install & Dev

```bash
pnpm install
pnpm tauri dev
```

> On first run, the SQL plugin creates the SQLite file in the OS-specific
> `ProjectDirs::data_dir` (e.g. macOS `~/Library/Application Support/dev.filename.shalgalt-omr/`,
> Linux `~/.local/share/dev.filename.shalgalt-omr/`) and applies migrations automatically.

## Project Layout (summary)

```
docs/             — BLUEPRINT, ARCHITECTURE
src/              — SvelteKit (routes, lib/ipc, lib/db, lib/stores, lib/components, lib/types)
src-tauri/
  migrations/     — SQL files embedded into Rust via `include_str!`
  src/
    domain/       — pure models (Rule 3 serialization unit)
    scan/         — CV pipeline (P2)
    grading/      — grading engine (P3)
    export/       — xlsx (P4)
    api/          — axum background server (Rule 4)
    commands/     — tauri::command IPC thin layer (Rule 1·2)
```

## License

MIT
