# Project Blueprint: Local-First OMR Grading IDE

## 1. Project Overview

본 프로젝트는 인터넷 연결이 불안정하고 저사양 PC가 주를 이루는 환경에서도 안정적으로 구동되는 '로컬 우선(Local-First) 데스크톱 OMR 채점 애플리케이션'입니다. 교사가 직접 OMR 양식을 에디터로 제작하고, 스캔된 대용량 PDF를 한 번에 불러와 컴퓨터 비전(Computer Vision)으로 오프라인 채점을 수행합니다. 채점 결과는 로컬 DB에 저장되며, 엑셀로 내보내거나 백그라운드 API 서버를 통해 외부 대시보드와 연동할 수 있어야 합니다.

## 2. Tech Stack

- **Frontend (UI & Editor):**
  - SvelteKit (SSG Mode, adapter-static 사용)
  - TailwindCSS
  - svelte-konva: HTML5 Canvas 기반의 OMR 버블 및 마커 드래그 앤 드롭 시각적 에디터 구현.
  - paneforge: IDE 형태의 패널 크기 조절 및 레이아웃 분할 구현.
- **Backend (Core & Processing):**
  - Tauri 2.0 + Rust
  - opencv-rust: 이미지 전처리, 투시 변환(Perspective Transform), 외곽선 검출 및 버블 판독 처리.[1]
  - pdfium-render: 다중 페이지 PDF 파일을 읽어 고해상도 개별 JPEG/PNG 이미지 버퍼로 분할.
  - rust_xlsxwriter: 다중 워크시트 및 셀 서식이 포함된 고성능 엑셀 파일 생성.
- **Database & API Server:**
  - sqlx + SQLite: 로컬 시스템에 학생 정보 및 채점 결과를 비동기적으로 안전하게 저장.[2]
  - axum: Tauri 앱 실행 시 백그라운드 스레드에서 RESTful API 서버(localhost:8080)를 구동하여 향후 웹사이트와의 데이터 연동 제공.

## 3. Core Architecture & Strict Rules (AI 주의 사항)

### Rule 1: IPC 메모리 누수 방지 (Memory Mirage Avoidance)

Tauri의 프론트엔드와 백엔드 간 통신(IPC) 시, 절대 대용량 이미지 데이터를 Base64로 인코딩하여 전송하지 마세요. 이는 직렬화 오버헤드와 엄청난 메모리 스파이크를 유발하여 앱을 다운시킵니다.[3]

- 프론트엔드는 스캔된 파일의 **로컬 절대 경로(Path) 문자열**만 Rust 코어로 전송해야 합니다.
- Rust에서 처리된 결과 이미지는 임시 폴더에 저장한 뒤, Tauri의 사용자 정의 프로토콜(`asset://localhost/`)을 통해 프론트엔드 HTML `<img>` 태그에서 렌더링하도록 구현하세요.

### Rule 2: 대용량 일괄 처리 시 UI Freezing 방지

수백 장의 OMR PDF를 분해하고 OpenCV로 채점하는 작업은 무거운 연산입니다.

- 모든 비전 연산과 DB 저장은 Rust의 `tokio::spawn`을 이용해 백그라운드 비동기 스레드에서 실행해야 합니다.
- 백그라운드 연산 중 `window.emit("task-progress", &progress)`를 사용하여 프론트엔드의 Svelte 스토어로 진행률(Progress) 이벤트를 지속적으로 쏴주어 프로그래스 바를 업데이트해야 합니다.

### Rule 3: OMR JSON 템플릿 직렬화

시각적 에디터(svelte-konva)에서 교사가 마우스로 배치한 4모서리의 기준점, 학번란, 문제 버블들의 X, Y 좌표와 정답 맵핑 데이터는 반드시 `template.json` 구조로 직렬화하여 SQLite 데이터베이스나 로컬 파일로 저장해야 합니다. OpenCV 알고리즘은 이 JSON 데이터를 불러와 스캔된 이미지의 픽셀 밀도 계산에 활용합니다.

### Rule 4: Axum API 서버의 독립성

axum 웹 서버 로직은 Tauri의 메인 UI 스레드를 막지 않도록 독립적으로 실행되어야 하며, 외부 웹사이트(클라우드 등)에서 데이터를 Fetch할 수 있도록 적절한 CORS 허용 정책을 구성해야 합니다.

## 4. SQLite Database Schema Guide

앱 초기화 시 sqlx 마이그레이션을 통해 다음 구조의 테이블을 생성하도록 설계하세요 [2]:

1. **students**: `id`, `name`, `grade`, `class`, `roll_number` (학번)
2. **templates**: `id`, `title`, `json_schema` (OMR 좌표 및 정답 키를 담은 TEXT 컬럼), `created_at`
3. **results**: `id`, `student_id` (FK), `template_id` (FK), `total_score`, `detail_answers` (JSON TEXT), `image_path` (채점 완료된 결과 이미지 로컬 경로), `created_at`

## 5. First Task for AI

위 명세서를 기반으로, 먼저 Tauri v2 + SvelteKit 초기 세팅을 진행하고, `src-tauri/Cargo.toml`에 필요한 Rust 의존성(opencv, pdfium-render, sqlx, axum, rust_xlsxwriter 등)을 추가하는 코드와 초기 폴더 구조 아키텍처를 제시해 주세요.
