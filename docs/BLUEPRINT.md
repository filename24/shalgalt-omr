# Project Blueprint: Local-First OMR Grading IDE

## 1. Project Overview

This project is a Local-First desktop OMR grading application designed to run reliably on
low-spec PCs and in environments where internet connectivity is unreliable. Teachers create
OMR forms directly inside the in-app editor, load large scanned PDFs in bulk, and grade
them offline with computer vision. Results are persisted to a local database, can be
exported to Excel, and can be served through a background API server for downstream
dashboards.

## 2. Tech Stack

- **Frontend (UI & Editor):**
  - SvelteKit (SSG mode via `adapter-static`).
  - TailwindCSS.
  - svelte-konva: HTML5-Canvas-based visual editor for OMR bubbles and markers, with
    drag-and-drop interaction.
  - paneforge: IDE-style resizable / splittable panel layout.
- **Backend (Core & Processing):**
  - Tauri 2.0 + Rust.
  - opencv-rust: image preprocessing, perspective transform, contour detection, and bubble
    reading.
  - pdfium-render: read multi-page PDF files and split them into high-resolution per-page
    JPEG/PNG image buffers.
  - rust_xlsxwriter: high-performance Excel output with multiple worksheets and cell
    formatting.
- **Database & API Server:**
  - `tauri-plugin-sql` + SQLite: store student records and grading results locally with
    asynchronous, frontend-driven access.
  - axum: when the Tauri app starts, a background-thread RESTful API server runs at
    `localhost:8080` to enable downstream dashboard integrations.

## 3. Core Architecture & Strict Rules (AI Guardrails)

### Rule 1 — IPC Memory Mirage Avoidance

When passing data across the Tauri frontend / backend IPC boundary, NEVER Base64-encode
large image data. Doing so causes serialization overhead and severe memory spikes that
crash the app.

- The frontend transmits only the **local absolute file path string** of the scanned file
  to the Rust core.
- Result images produced by the Rust core are written to a temp directory and rendered in
  the frontend through Tauri's custom protocol (`asset://localhost/`) using a plain HTML
  `<img>` tag.

### Rule 2 — No UI Freeze on Batch Work

Decomposing hundreds of OMR PDF pages and grading them with OpenCV is a heavy operation.

- Every CV operation and DB write MUST run on a `tokio::spawn` background task.
- During background execution, emit `window.emit("task-progress", &progress)` events
  continuously so that the frontend Svelte store can keep the progress bar updated.

### Rule 3 — OMR JSON Template Serialization

The reference markers (4 corners), student-id area, and per-question bubble coordinates
that the teacher places in the visual editor (svelte-konva), along with the answer-key
mapping, MUST be serialized as a `template.json` structure and persisted into SQLite or a
local file. The OpenCV algorithm reads this JSON to compute pixel density on the scanned
image.

### Rule 4 — axum API Server Independence

The axum server logic MUST run independently so that it does not block the Tauri main UI
thread. It must apply an appropriate CORS policy so that external websites (cloud
dashboards, etc.) can fetch data.

## 4. SQLite Database Schema Guide

On first start, the SQL plugin's migrations create the following tables:

1. **students**: `id`, `name`, `grade`, `class`, `roll_number`.
2. **templates**: `id`, `title`, `json_schema` (TEXT — OMR coordinates and answer key),
   `created_at`.
3. **results**: `id`, `student_id` (FK), `template_id` (FK), `total_score`,
   `detail_answers` (JSON TEXT), `image_path` (absolute path to the graded result image),
   `created_at`.
