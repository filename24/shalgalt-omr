# `crates/shalgalt-fileformat` — `.shalgalt` Project File (P4)

Read/write the `.shalgalt` project-file format: a zip container with optional `age`
passphrase encryption. Used to share entire exam projects (template + answer key +
graded sheets + roster) between teachers across machines.

> Status: **implemented** (P4-01/02). The crate streams a zip container with optional `age`
> passphrase encryption. Decisions are recorded in
> [`docs/adr/0011-shalgalt-file-format.md`](../../docs/adr/0011-shalgalt-file-format.md) and
> [`docs/adr/0012-age-encryption.md`](../../docs/adr/0012-age-encryption.md). The Tauri command
> wiring (`project_open` / `project_save` / `project_export`) lands separately in P4-03.

> Repo-level rules, language conventions, and locked decisions live in
> [`/AGENTS.md`](../../AGENTS.md). This file describes only what is specific to this crate.

## Locked Decisions (master plan §6.4)

- **Container**: zip. Single file, recognizable extension, opens in any zip viewer for
  inspection (when unencrypted).
- **Encryption**: optional, `age` with passphrase recipient (RFC 9580-ish format from
  `str4d/rage`). When encryption is on, every file inside the zip is encrypted EXCEPT
  the manifest.
- **`manifest.json` is always plaintext**, even when the rest is encrypted. This lets the
  desktop app preview a project (title, exam date, sheet count) without prompting for a
  passphrase. The manifest MUST NOT contain student PII.
- **Magic header**: zip's own `PK\x03\x04` is sufficient. No custom prefix.
- **Versioning**: `manifest.json` carries a `format_version` integer. Loaders MUST refuse
  versions newer than they understand and emit `AppError { code: "fileformat.version_too_new" }`.

## Module Layout (as implemented, P4-01/02)

```
src/
  lib.rs              — Module re-exports + `FORMAT_VERSION` / `MANIFEST_ENTRY` consts.
                        Public surface: free functions `open` / `open_manifest_only` / `write`
                        plus `Manifest`, `ReadHandle`, `EntryIter`, `FileFormatError`. There is
                        no `ProjectFile` facade — the free functions cover every caller and the
                        command layer (P4-03) builds its own domain helpers on top.
  manifest.rs         — `Manifest { format_version, title, created_at, sheet_count, encrypted,
                        exam_id?, hint? }` + `from_reader` (runs the version gate).
  reader.rs           — Streaming zip reader. `open` parses the manifest, then `EntryIter`'s
                        lending `next_entry()` yields one borrowed `ReadHandle` at a time,
                        decrypting on demand. (Not `impl Iterator`: `Item` cannot hold the
                        per-entry archive borrow.)
  writer.rs           — Streaming zip writer. Manifest first (plaintext), then each entry
                        age-wrapped on the fly when a passphrase is set.
  crypto.rs           — `pub(crate)` thin wrapper over `age` (the ONLY module that touches the
                        age API) so the rest of the crate stays crypto-library-agnostic.
  error.rs            — `FileFormatError` (thiserror) + a public `code() -> &'static str`.
tests/
  roundtrip.rs        — Write → read → assert content equality (encrypted + plaintext), 1 MiB
                        streamed-from-path entry, deterministic order, manifest-no-PII.
  bad_passphrase.rs   — Wrong / missing passphrase → `FileFormatError::BadPassphrase`; manifest
                        still readable without a passphrase.
  forward_compat.rs   — `format_version = 999` → refused with `fileformat.version_too_new`,
                        before any decryption.
```

### Stable error codes (locked — the desktop string-table keys to them)

`FileFormatError::code()` returns one of: `fileformat.version_too_new`,
`fileformat.bad_passphrase`, `fileformat.malformed`, `fileformat.io`, `fileformat.serde`. The
wrong-passphrase variant is `BadPassphrase` (fieldless, never leaks age internals) — note this
supersedes the earlier draft name `AuthenticationFailed`.

## Critical Rules — fileformat

### Rule 1 — Streaming, never load whole file into memory

The crate MUST stream both reads and writes. A `.shalgalt` project can contain hundreds
of MB of scanned PDFs; loading the whole archive into a `Vec<u8>` would re-introduce the
"memory mirage" Rule 1 explicitly forbids in the IPC layer.

Concretely:

- `reader::open` returns an iterator over `(name, ReadHandle)`. Callers consume each
  entry to a destination `Path` without holding the whole zip in memory.
- `writer::write` takes an iterator of `(name, ReadHandle)` and streams encrypted bytes
  through to the output zip.

### Manifest contract

`manifest.json` keys (all required):

```json
{
  "format_version": 1,
  "title": "string",
  "created_at": "RFC 3339 timestamp",
  "sheet_count": 0,
  "encrypted": false
}
```

`encrypted: true` means the rest of the archive entries are age-wrapped. The manifest
itself is always plaintext.

## What does NOT belong here

- The `OmrTemplate` struct or grading types — those live in `shalgalt-core::domain`.
- PDF rendering — `shalgalt-pdf` produces the PDF bytes that go into the archive.
- Tauri / IPC code — the desktop app's `commands::` module is the only place that
  bridges `tauri::command` invocations to this crate.
- Telemetry, crash reporting, analytics — repo policy.
