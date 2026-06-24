# Build & run (https://filename24.github.io/shalgalt-omr/en/docs/dev/build)



shalgalt-omr depends on native libraries (OpenCV for the CV pipeline, pdfium at runtime,
plus the platform webview toolkit for Tauri). Install the system prerequisites first, then
the JS and Rust toolchains.

## System prerequisites [#system-prerequisites]

<Tabs items="[&#x22;Debian / Ubuntu&#x22;, &#x22;macOS&#x22;, &#x22;Windows&#x22;]">
  <Tab value="Debian / Ubuntu">
    ```bash title="Debian / Ubuntu prerequisites"
    sudo apt update
    sudo apt install -y \
      build-essential pkg-config libssl-dev \
      libopencv-dev clang libclang-dev \
      libgtk-3-dev libwebkit2gtk-4.1-dev \
      libayatana-appindicator3-dev librsvg2-dev
    ```
  </Tab>

  <Tab value="macOS">
    ```bash title="macOS (Homebrew) prerequisites"
    brew install opencv pkg-config llvm
    ```
  </Tab>

  <Tab value="Windows">
    Per [ADR 0001](/adr/0001-windows-opencv-strategy) the project uses the upstream OpenCV
    prebuilt self-extractor. LLVM (for opencv-rust's bindgen) ships with recent Visual Studio
    installs; otherwise install it from llvm.org.

    ```powershell title="Windows OpenCV setup"
    # 1) Download and extract OpenCV to C:\tools\opencv
    $url = "https://github.com/opencv/opencv/releases/download/4.10.0/opencv-4.10.0-windows.exe"
    Invoke-WebRequest $url -OutFile opencv.exe
    Start-Process .\opencv.exe -ArgumentList '-o"C:\tools" -y' -Wait
    Remove-Item opencv.exe

    # 2) Set env vars (persist with setx or via your shell profile)
    $env:OPENCV_LINK_LIBS     = "opencv_world4100"
    $env:OPENCV_LINK_PATHS    = "C:\tools\opencv\build\x64\vc16\lib"
    $env:OPENCV_INCLUDE_PATHS = "C:\tools\opencv\build\include"
    $env:LIBCLANG_PATH        = "C:\Program Files\LLVM\bin"

    # 3) Put the OpenCV runtime DLL directory on PATH
    $env:Path = "C:\tools\opencv\build\x64\vc16\bin;$env:Path"
    ```

    <Callout type="warn" title="Bumping OpenCV">
      Bumping OpenCV means updating three places: the snippet above, the `OPENCV_VERSION`
      job-level env in `.github/workflows/ci.yml`, and
      [ADR 0001](/adr/0001-windows-opencv-strategy).
    </Callout>
  </Tab>
</Tabs>

### pdfium (runtime, all platforms) [#pdfium-runtime-all-platforms]

`pdfium-render` looks up the pdfium dynamic library at runtime. Download a release for your
OS from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) and place
it next to the executable, or extend `LD_LIBRARY_PATH` (Linux) / `DYLD_LIBRARY_PATH` (macOS)
/ `PATH` (Windows). Release bundles ship pdfium as a bundled resource.

## Toolchains [#toolchains]

* **Node** ≥ 20 and **pnpm** (the repo pins versions via `package.json` / `pnpm-lock.yaml`).
* **Rust** stable (`rustup`), with `cargo fmt` and `clippy` components.

## Common commands [#common-commands]

| Command               | Purpose                                                           |
| --------------------- | ----------------------------------------------------------------- |
| `pnpm install`        | Install JS dependencies.                                          |
| `pnpm dev`            | SvelteKit dev server only (browser, no Tauri webview).            |
| `pnpm tauri dev`      | Native window + Rust core (auto-runs Vite). The default flow.     |
| `pnpm check`          | `svelte-kit sync` + `svelte-check`.                               |
| `pnpm build`          | Build SvelteKit assets (consumed by `tauri build`).               |
| `pnpm tauri build`    | Production native bundle.                                         |
| `pnpm generate-types` | Regenerate TS bindings from Rust (`cargo test -p shalgalt-core`). |

Run these at the workspace root for the Rust side:

```bash title="Rust workspace checks"
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test --workspace
```

## First run [#first-run]

<Callout type="info" title="SQLite is created automatically">
  On first launch the SQL plugin creates the SQLite file in the OS-specific data directory
  (e.g. Linux `~/.local/share/dev.filename.shalgalt-omr/`, macOS
  `~/Library/Application Support/dev.filename.shalgalt-omr/`) and applies migrations
  automatically.
</Callout>

## Running the standalone HTTP server [#running-the-standalone-http-server]

```bash title="Run shalgalt-server"
export SHALGALT_API_TOKEN="a-strong-random-token"
cargo run -p shalgalt-server -- --db /path/to/db.sqlite
```

See the [HTTP API reference](/dev/api) for CLI flags, bind address, and auth behaviour.
