# ADR 0011 — `.shalgalt` project file (https://filename24.github.io/shalgalt-omr/en/docs/adr/0011-shalgalt-file-format)



<Callout type="success" title="Accepted · 2026-05-31">
  **Deciders:** filename24 · **Related:** P4-01 (`shalgalt-fileformat` crate skeleton),
  P4-03 (Tauri commands) · **Related ADRs:** [0012](/adr/0012-age-encryption) (optional
  `age` encryption layer).
</Callout>

## Context [#context]

A teacher's exam project is not a single file. It is a template (the `OmrTemplate`, Rule 3),
one answer key per variant, free-form metadata (school, subject, exam date), an optional
student roster, and optionally the original test-paper PDF. Master plan §2 calls for sharing
a whole project between teachers across machines as **one** artifact — "juggling multiple
files" is one of the pain points the product exists to remove (§4).

We need a container format that:

1. Bundles many heterogeneous files into one.
2. Can be **previewed** (title, exam date, sheet count) without opening or decrypting the
   whole thing — the dashboard and the import dialog show project cards.
3. Streams. A project can carry hundreds of MB of scanned PDFs; Rule 1 ("IPC Memory Mirage
   Avoidance") forbids loading large payloads whole, and that discipline must extend to the
   archive layer or it is pointless.
4. Is inspectable with off-the-shelf tools when unencrypted, so a stuck teacher (or a support
   engineer) can open it without our app.

## Options considered [#options-considered]

| Option                                   | Bundles many files | Cheap metadata preview                                | Streaming read/write                | Off-the-shelf inspectable           | Notes                                                                                   |
| ---------------------------------------- | ------------------ | ----------------------------------------------------- | ----------------------------------- | ----------------------------------- | --------------------------------------------------------------------------------------- |
| Single JSON blob (base64 the PDF inside) | yes                | no — must parse the whole blob                        | no — whole file in memory           | poor (giant base64)                 | Violates Rule 1 outright.                                                               |
| SQLite file as the container             | yes                | yes (a metadata table)                                | partial (blob streaming is awkward) | needs a SQLite viewer               | Overkill; we already have a SQLite *app* DB and do not want a second schema to version. |
| `tar` archive                            | yes                | no — no central directory; must scan sequentially     | yes                                 | yes                                 | Cannot seek to the manifest; preview means reading from the front.                      |
| **zip archive**                          | yes                | **yes — central directory + a named `manifest.json`** | **yes — per-entry `Read`/`Write`**  | **yes — every OS has a zip viewer** | Chosen.                                                                                 |
| Custom binary format                     | yes                | yes                                                   | yes                                 | no — bespoke tooling                | Reinvents zip with no upside and a parser to maintain.                                  |

## Decision [#decision]

<Callout type="info" title="Decision">
  Adopt a **zip container** with the extension `.shalgalt`, an always-plaintext
  `manifest.json` preview surface, and a fixed internal layout.
</Callout>

```
example.shalgalt   (zip)
├── manifest.json          # always plaintext — the preview surface
├── template.json          # OmrTemplate (Rule 3)
├── answer-keys.json       # { "A": [...], "B": [...], ... }
├── metadata.json          # school, teacher, subject, exam_date, notes
├── students.csv           # optional roster
└── exam.pdf               # optional original test paper
```

Key sub-decisions:

* **`manifest.json` is always plaintext**, even when the rest of the archive is encrypted
  (see [0012](/adr/0012-age-encryption)). It is the only entry a reader may buffer whole — it
  is tiny. It MUST NOT contain student PII, because anyone can read it without the passphrase.
  Its required keys are `format_version`, `title`, `created_at` (RFC 3339), `sheet_count`,
  `encrypted`; `exam_id` and `hint` are optional.
* **No custom magic header.** zip's own `PK\x03\x04` signature plus the `.shalgalt` extension
  is sufficient to recognize the format. A custom prefix would break "opens in any zip viewer."
* **`format_version` integer gates forward compatibility.** A loader refuses any manifest whose
  `format_version` exceeds the version it understands, returning the stable code
  `fileformat.version_too_new`. The check runs immediately after the manifest is parsed, before
  any decryption — a future-format file fails with a precise reason rather than a confusing
  crypto or zip error.
* **Streaming is mandatory (Rule 1).** The `shalgalt-fileformat` crate exposes a `write` that
  takes an iterator of `(name, ReadHandle)` and an `open` that returns a lending iterator
  (`EntryIter::next_entry`) yielding one borrowed `ReadHandle` at a time. Nothing but the
  manifest is ever held whole in memory. `ReadHandle::from_path` lets a caller feed a
  multi-hundred-MB `exam.pdf` straight from disk.
* **Compression policy:** small plaintext JSON/CSV is DEFLATE-compressed; entries that do not
  benefit (already-compressed PDFs, and high-entropy age ciphertext) are `Stored`. flate2's
  pure-Rust `zlib-rs` backend is used so there is no system C `zlib` build dependency.

## Consequences [#consequences]

* A new pure-Rust crate `crates/shalgalt-fileformat` owns read/write. It depends on `zip`
  (no default features — the bzip2/zstd/xz/aes C codecs are dropped) and, for the encrypted
  case, `age`. It deliberately does **not** depend on `shalgalt-core`: it streams opaque named
  byte blobs and stays domain-agnostic, so the `OmrTemplate`/`AnswerKey` types remain solely
  in `shalgalt-core::domain`.
* The public API is a set of free functions (`open`, `open_manifest_only`, `write`) plus
  `Manifest` and `ReadHandle`, rather than a `ProjectFile` facade. The earlier placeholder doc
  sketched `ProjectFile::open/write`; the free-function shape is simpler for every P4 caller
  and the Tauri command layer (P4-03) builds its own domain helpers on top. A facade can be
  added later without breaking these signatures.
* Entry names are validated against zip-slip (absolute paths, `..`, backslashes) on both the
  write and read paths; `open` resolves names via `enclosed_name` and rejects anything unsafe.
* The desktop command layer (P4-03) maps `FileFormatError::code()` to the Mongolian string
  table the same way it already maps `AppError::code`.

## Follow-up [#follow-up]

* [0012](/adr/0012-age-encryption) records the optional `age` passphrase encryption layered on
  top of this container.
* P4-03 wires `project_open` / `project_save` / `project_export` Tauri commands onto this
  crate and adds the `$lib/ipc/project.ts` frontend wrapper.
* P4-07 registers the `.shalgalt` extension with the OS so a double-click opens the app.
* A future `format_version: 2` (if the layout ever changes) must ship with a migration note and
  a loader that still reads version 1.

<Cards>
  <Card href="/adr/0012-age-encryption" title="ADR 0012 — Optional age encryption">
    The optional `age` passphrase encryption layered on top of this container.
  </Card>
</Cards>
