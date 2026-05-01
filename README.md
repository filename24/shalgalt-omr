# shalgalt-omr

> Local-First OMR Grading IDE — Tauri 2 + SvelteKit + Rust.
>
> 상세 명세는 [`docs/BLUEPRINT.md`](docs/BLUEPRINT.md), 폴더 구조와 모듈 책임은
> [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) 참조.

## Stack

| Layer       | Tech                                                                  |
| ----------- | --------------------------------------------------------------------- |
| Frontend    | SvelteKit 5 (adapter-static), TailwindCSS v4, svelte-konva, paneforge |
| Native UI   | Tauri 2.0                                                             |
| Core        | Rust — opencv-rust, pdfium-render, rust_xlsxwriter                    |
| Persistence | sqlx + SQLite (local file)                                            |
| API         | axum (background 8080)                                                |

## System Prerequisites

본 프로젝트는 네이티브 의존성이 있어 빌드 전에 시스템 패키지가 설치되어 있어야 합니다.

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

[vcpkg로 OpenCV](https://github.com/microsoft/vcpkg) 설치 후 `OPENCV_LINK_LIBS`/`OPENCV_INCLUDE_PATHS`
환경변수 설정. 자세한 절차는 P0 작업 후 ADR로 추가 예정.

### pdfium

`pdfium-render`는 런타임에 pdfium 동적 라이브러리를 찾습니다.
[bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) 릴리스에서 OS에 맞는 바이너리를 받아
실행 파일과 같은 폴더 또는 `LD_LIBRARY_PATH` (Linux), `DYLD_LIBRARY_PATH` (macOS), `PATH` (Windows) 에 둡니다.
P4 단계에서 `tauri.conf.json#bundle.resources`에 자동 포함하는 스크립트를 추가합니다.

## Install & Dev

```bash
pnpm install
pnpm tauri dev
```

> 처음 실행 시 sqlx가 OS 별 `ProjectDirs::data_dir`(예: macOS
> `~/Library/Application Support/dev.filename.shalgalt-omr/`, Linux
> `~/.local/share/dev.filename.shalgalt-omr/`)에 SQLite 파일을 만들고
> 마이그레이션을 자동 적용합니다.

## Project Layout (요약)

```
docs/             — BLUEPRINT, ARCHITECTURE
src/              — SvelteKit (routes, lib/ipc, lib/stores, lib/components, lib/types)
src-tauri/
  migrations/     — sqlx 마이그레이션
  src/
    domain/       — 순수 모델 (Rule 3 직렬화 단위)
    db/           — Repository 패턴
    scan/         — CV 파이프라인 (P2)
    grading/      — 채점 엔진 (P3)
    export/       — xlsx (P4)
    api/          — axum 백그라운드 서버 (Rule 4)
    commands/     — tauri::command IPC 박막 (Rule 1·2)
```

## License

MIT
