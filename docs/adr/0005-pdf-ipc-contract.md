# ADR 0005 — PDF generation IPC contract

- **Status**: Accepted
- **Date**: 2026-05-08
- **Deciders**: filename24
- **Implements**: P2-07 / P2-08

## Context

`shalgalt-pdf::render_template` returns a complete PDF byte stream. The editor needs
to (a) write that stream to a user-picked path on disk, and (b) display a rasterized
preview of it inside the editor pane without round-tripping the bytes through the
webview. Doing this naïvely tempts two anti-patterns:

1. Sending the rendered PDF bytes through `invoke()` as a `Vec<u8>` or, worse, a
   base64 string — millions of bytes per render, locking the IPC bridge for hundreds
   of milliseconds.
2. Embedding the PNG preview inside the same response payload as a data URL — same
   cost, plus inflates webview memory.

Master plan §6 fixes Rule 1 ("paths only over IPC") at the architecture level. This
ADR records exactly *how* the new PDF commands honor the rule.

## Decision

Two `tauri::command` handlers ship in `apps/desktop/src/commands/pdf.rs`:

```rust
#[tauri::command]
pub async fn pdf_generate_omr(
    _state: State<'_, AppState>,
    template_json: String,
    variant: Option<String>,
    output_path: String,
) -> AppResult<()>;

#[tauri::command]
pub async fn pdf_render_template_preview(
    state: State<'_, AppState>,
    template_json: String,
    variant: Option<String>,
) -> AppResult<String>;
```

Conventions:

- `template_json` is the serialized `OmrTemplate` (Rule 3 unit). Keeps the IPC
  payload uniform with how the frontend already serializes for the SQL plugin.
- `output_path` is the absolute path returned by the native save dialog
  (`pickPdfSavePath`). The save-dialog scope grants Tauri write permission for
  exactly that path even when it is outside the default `$APPDATA` capability.
- `pdf_render_template_preview` writes to `<APPCACHE>/preview/<sha256>.pdf`, then
  rasterizes the first page via the existing `shalgalt_cv::preview::rasterize_first_page`
  helper. Returns the absolute PNG path, which the frontend feeds through
  `convertFileSrc(...)` to load via the `asset://localhost/` protocol.
- Both commands wrap the synchronous `printpdf` call in `tokio::task::spawn_blocking`
  (Rule 2), and both write to disk via `tokio::fs::write` so the async runtime stays
  free for IPC traffic.
- Errors map to existing `AppError` variants (`BadRequest` for empty/invalid input,
  `Internal` for filesystem and render failures). New failure codes were considered
  but rejected: the frontend toast surface already covers the user-facing shape, and
  adding error variants increases the API surface every consumer has to translate.
- Cache hygiene: `prune_old_previews` runs on app startup and drops `<APPCACHE>/preview/*`
  files older than 24 hours. Cleanup is best-effort; failure logs but never panics.

## Consequences

**Positive**

- IPC payloads stay below 100 KB even for the densest preset.
- The rasterized preview reuses the asset-protocol helper introduced in P1, so we
  avoid a second image-loading code path in the webview.
- `pdf_generate_omr` is reused by P5's "answer-key proof PDF" feature without
  changes — the variant flag is already accepted.

**Negative**

- The cache directory grows linearly until pruning runs at next app launch. For
  power users editing huge templates this can briefly hit dozens of MB. Acceptable
  for a desktop app; not acceptable on mobile (out of scope).
- `template_json` parsing happens on every preview render. For very large templates
  this is measurable (~1 ms on a midrange laptop for a 51-group preset); replacing
  it with a typed payload would require ts-rs-aware Tauri parameter binding, which
  is not currently supported by upstream.

## Follow-up

- If the parsing cost ever shows up in profiling, switch the param to
  `template: OmrTemplate` once `tauri::command` learns to derive its parameter types
  from `ts-rs`-derived structs.
- Multi-variant batch export (P5) reuses this command — the `variant` parameter is
  already plumbed through to `PdfOptions::variant`.
