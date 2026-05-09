# `crates/shalgalt-fileformat` — `.shalgalt` Project File (P4)

Read/write the `.shalgalt` project-file format: a zip container with optional `age`
passphrase encryption. Used to share entire exam projects (template + answer key +
graded sheets + roster) between teachers across machines.

> Status: **placeholder**. The current `src/lib.rs` is one comment; the real
> implementation lands in P4 per the master plan §10. This file documents the locked
> decisions so the implementer in P4 has a clear contract to honor.

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

## Expected Module Layout (P4)

```
src/
  lib.rs              — `ProjectFile::open(path, passphrase) -> ProjectFile`
                        and `ProjectFile::write(path, …)` entry points.
  manifest.rs         — `Manifest { format_version, title, created_at, sheet_count, encrypted }`.
  reader.rs           — Streaming zip reader. Decrypts entries on demand via age.
  writer.rs           — Streaming zip writer. Encrypts entries when a passphrase is set.
  crypto.rs           — Thin wrapper over `age` so the rest of the crate stays
                        crypto-library-agnostic.
  error.rs            — `FileFormatError` (thiserror).
tests/
  roundtrip.rs        — Write → read → assert content equality (encrypted + plaintext).
  bad_passphrase.rs   — Wrong passphrase → `FileFormatError::AuthenticationFailed`.
  forward_compat.rs   — `format_version = 999` → refused with the right error code.
```

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
