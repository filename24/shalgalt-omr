# shalgalt-omr — Initial Architecture (P0)

> 본 문서는 BLUEPRINT.md §5 "First Task"에 따라 P0 단계의 폴더 구조와 모듈 책임을 정의한다. 후속 Phase에서 모듈이 추가/세분화될 수 있다.

> **DB 접근 정책 (확정).** SQLite 접근은 [`tauri-plugin-sql`](https://v2.tauri.app/ko/plugin/sql/)로 일원화한다.
> 마이그레이션은 Rust 측에서 `Migration { version, sql, kind: Up }` 으로 등록되고, 프론트엔드는
> `@tauri-apps/plugin-sql` 의 `Database.load("sqlite:shalgalt-omr.sqlite")` 로 접근한다.
> Rust 측에는 별도의 `sqlx` 풀을 두지 않는다. 따라서 `commands/` 모듈에는 DB read/write 명령이 없고
> CV/PDF/xlsx 등 **Rust 전용 무거운 작업**만 IPC로 노출된다.

## 1. Top-level Layout

```
shalgalt-omr/
├── docs/                          # 설계 문서 (BLUEPRINT, ARCHITECTURE, 추후 ADR)
├── src/                           # SvelteKit (SSG) 프론트엔드
├── src-tauri/                     # Rust 코어 (Tauri 2.0)
├── static/                        # 정적 자산
├── package.json
├── pnpm-lock.yaml
├── svelte.config.js
├── vite.config.js
├── tsconfig.json
├── tailwind.config.ts             # P0 추가
└── postcss.config.js              # P0 추가
```

## 2. Frontend (`src/`)

```
src/
├── app.html
├── app.css                        # Tailwind directives + CSS 변수
├── lib/
│   ├── ipc/                       # Rust 전용 명령 wrapper (스캔, 엑셀)
│   │   ├── index.ts
│   │   ├── scan.ts                # PDF 분해 / 채점 invoke + emit listener
│   │   └── export.ts              # xlsx 내보내기 invoke
│   ├── db/                        # tauri-plugin-sql Repository 계층
│   │   ├── index.ts               # Database.load 캐시 (sqlite:shalgalt-omr.sqlite)
│   │   ├── templates.ts
│   │   ├── results.ts
│   │   └── students.ts
│   ├── stores/                    # Svelte 5 runes 기반 상태
│   │   ├── progress.svelte.ts     # task-progress emit 구독
│   │   ├── currentTemplate.svelte.ts
│   │   └── session.svelte.ts
│   ├── components/
│   │   ├── shell/                 # paneforge IDE 셸
│   │   │   ├── IdeShell.svelte    # 좌(트리) / 중(작업영역) / 우(인스펙터)
│   │   │   ├── ActivityBar.svelte
│   │   │   └── StatusBar.svelte
│   │   ├── editor/                # svelte-konva 에디터
│   │   │   ├── TemplateCanvas.svelte
│   │   │   ├── MarkerLayer.svelte         # 4 코너 기준점
│   │   │   ├── BubbleGroupLayer.svelte    # 학번/문항 버블 그룹
│   │   │   ├── PropertyPanel.svelte
│   │   │   └── Toolbar.svelte
│   │   ├── grader/                # 일괄 채점 UI
│   │   │   ├── PdfPicker.svelte
│   │   │   ├── ProgressOverlay.svelte
│   │   │   └── ReviewGrid.svelte          # 인식 실패 페이지 수동 정정
│   │   ├── results/
│   │   │   ├── ResultTable.svelte
│   │   │   └── ResultDetail.svelte
│   │   └── ui/                    # 공통 원자 컴포넌트
│   │       ├── Button.svelte
│   │       └── Dialog.svelte
│   └── types/                     # 백엔드와 공유되는 모양 (수동 정의)
│       ├── template.ts            # OmrTemplate, BubbleGroup, Marker
│       ├── result.ts
│       └── progress.ts
└── routes/                        # SvelteKit App Router
    ├── +layout.svelte             # IdeShell 래핑
    ├── +page.svelte               # 대시보드 (최근 템플릿/결과)
    ├── editor/
    │   └── +page.svelte           # 템플릿 에디터
    ├── grade/
    │   └── +page.svelte           # PDF 일괄 채점
    └── results/
        └── +page.svelte           # 결과 조회/내보내기
```

## 3. Rust Core (`src-tauri/`)

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── build.rs
├── capabilities/
│   └── default.json
├── migrations/                    # tauri-plugin-sql `Migration::sql` 로 임베드되는 SQL
│   └── 0001_init.sql              # `include_str!` 로 lib.rs 에서 로드
└── src/
    ├── main.rs                    # 단순히 lib::run() 호출
    ├── lib.rs                     # tauri::Builder + plugin-sql 등록 + setup hook
    ├── error.rs                   # AppError + AppResult — anyhow → frontend-safe
    ├── state.rs                   # AppState { dirs } (DB 풀 없음 — plugin-sql 이 담당)
    ├── paths.rs                   # AppDirs (data dir, scans dir, cache dir)
    ├── domain/                    # 순수 모델 (직렬화 가능, DB-agnostic)
    │   ├── mod.rs
    │   ├── template.rs            # OmrTemplate, BubbleGroup, Marker — Rule 3 직렬화 단위
    │   ├── student.rs
    │   └── result.rs
    ├── scan/                      # CV 파이프라인
    │   ├── mod.rs
    │   ├── pdf.rs                 # pdfium-render — PDF → Mat/DynamicImage 페이지 시퀀스
    │   ├── perspective.rs         # 4-corner detection + warpPerspective
    │   ├── bubbles.rs             # ROI 평균 픽셀 농도 → 마킹 판독
    │   └── pipeline.rs            # tokio::spawn 오케스트레이터, emit("task-progress")
    ├── grading/
    │   ├── mod.rs
    │   └── engine.rs              # template.json + parsed bubbles → score & detail
    ├── export/
    │   ├── mod.rs
    │   └── xlsx.rs                # rust_xlsxwriter
    ├── api/                       # axum 백그라운드 서버 (Rule 4 — 독립 task)
    │   ├── mod.rs
    │   ├── server.rs              # tokio::spawn으로 8080 listen, 종료 시 graceful shutdown
    │   ├── routes.rs              # GET /healthz, /api/results, /api/templates …
    │   └── cors.rs                # tower_http::cors 정책
    └── commands/                  # tauri::command 박막 — IPC 진입점 (Rust 전용 작업)
        ├── mod.rs
        ├── scan.rs                # 인자: 로컬 절대 경로 (Rule 1)
        └── export.rs
```

## 4. Module Boundaries

| Layer       | 의존 방향                          | 비고                                                                                  |
| ----------- | ---------------------------------- | ------------------------------------------------------------------------------------- |
| `domain`    | (의존 없음)                        | 순수 데이터. 어떤 인프라에도 의존하지 않음.                                           |
| `scan`      | `domain`                           | OpenCV/pdfium 호출. 결과는 `domain::ParsedSheet` 타입으로만 반환.                     |
| `grading`   | `domain`                           | 순수 함수: `(OmrTemplate, ParsedSheet) -> ScoreResult`. CV 호출 금지.                 |
| `export`    | `domain`                           | xlsx 직렬화.                                                                          |
| `api`       | `domain`, `state`                  | Tauri 윈도우와 무관. `AppState` clone만 받음.                                         |
| `commands`  | `domain`, `scan`, `grading`, `export` | IPC 박막. 검증 + 위 모듈 호출 + `AppError` 변환. DB 명령은 없음 (plugin-sql 사용).  |
| `lib.rs`    | `commands`, `api`, `state`         | 부팅 순서: tracing → AppDirs → plugin-sql(migrations) → axum spawn → tauri builder.   |
| Frontend `lib/db/` | `lib/types`                | `tauri-plugin-sql` 어댑터. SQL 직접 호출, `OmrTemplate` JSON 직렬화/역직렬화 책임.    |

## 5. IPC & Threading Rules (Blueprint §3 적용)

- **이미지는 Path 만 오간다.** `commands::scan::grade_pdf`는 `pdf_path: String`만 인자로 받음. 결과 이미지는 `app_dirs.scans_dir`에 저장 후 경로 반환.
- **모든 무거운 작업은 `tokio::spawn`.** `tauri::command async fn`은 즉시 task handle을 띄우고 await 하지 않은 채 반환 (혹은 `app_handle.emit("task-progress", ...)` 통해 통신).
- **진행률 이벤트 페이로드 표준화** — `domain::TaskProgress { task_id, processed, total, stage }`.
- **axum 서버**는 `setup` hook에서 `tokio::spawn`으로 띄우고, `AppHandle::on_window_event(CloseRequested)` 시 `oneshot` 셧다운 시그널.

## 6. SQLite Schema (P0)

`src-tauri/migrations/0001_init.sql`:

```sql
CREATE TABLE students (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  name            TEXT    NOT NULL,
  grade           INTEGER NOT NULL,
  class           INTEGER NOT NULL,
  roll_number     TEXT    NOT NULL,
  created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (grade, class, roll_number)
);

CREATE TABLE templates (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  title           TEXT    NOT NULL,
  json_schema     TEXT    NOT NULL,        -- OmrTemplate 직렬화 (Rule 3)
  created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE results (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  student_id      INTEGER REFERENCES students(id) ON DELETE SET NULL,
  template_id     INTEGER NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
  total_score     REAL    NOT NULL,
  detail_answers  TEXT    NOT NULL,        -- JSON: per-question 정/오/blank
  image_path      TEXT,                    -- 채점 결과 이미지 절대경로
  created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_results_template ON results(template_id);
CREATE INDEX idx_results_student  ON results(student_id);
```

## 7. Open Decisions (사용자 확정 필요)

| #   | 항목                  | 기본값 (P0 진행 시 가정)                                                                  |
| --- | --------------------- | ----------------------------------------------------------------------------------------- |
| 1   | OpenCV 의존성 전략    | `opencv` crate `clang-runtime` feature, 빌드 시 시스템 OpenCV(`libopencv-dev`) 요구       |
| 2   | pdfium 동적 라이브러리 | `pdfium-render` + `bblanchon/pdfium-binaries` 자동 다운로드 (CI/배포 단계에서 처리)       |
| 3   | 타깃 OS              | Windows / macOS / Linux 데스크톱 (Tauri 기본 cross)                                       |
| 4   | UI 언어              | 한국어 only (i18n 도입은 P5 polish에서)                                                   |
| 5   | Phase 진행 순서      | P0 → P1 → P2 → P3 → P4 → P5                                                               |

위 기본값으로 P0 코드 변경(다음 단계)을 시작합니다. 변경이 필요하면 알려주세요.
