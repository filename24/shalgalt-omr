---
title: ADR 0012 — Optional age encryption
description: Passphrase-based, ASCII-armored age encryption applied per-entry to .shalgalt payloads, leaving the manifest readable.
---

<Callout type="success" title="Accepted · 2026-05-31">
  **Deciders:** filename24 · **Related:** P4-02 (`age` passphrase encryption layer) ·
  **Related ADRs:** [0011](/adr/0011-shalgalt-file-format) (the `.shalgalt` zip container).
</Callout>

## Context

A `.shalgalt` project bundles student-identifying data: a roster (`students.csv`) and graded
results tied to names. When a teacher emails a project, drops it on a shared drive, or carries
it on a USB stick, that PII is exposed. We want **optional** at-rest encryption so a teacher can
protect a project with a passphrase, while keeping unencrypted projects trivially inspectable
for the common case.

Constraints:

1. **Passphrase, not key files.** Teachers are non-technical (master plan §7, "design for older
   teachers"). Asking them to manage X25519 key pairs is a non-starter. A single passphrase they
   choose and remember is the only usable model.
2. **The manifest must stay readable.** Per [0011](/adr/0011-shalgalt-file-format), the dashboard
   previews `manifest.json` without a passphrase. So encryption must be **per-entry**, wrapping
   every file *except* the manifest — not a whole-archive blob.
3. **Streaming (Rule 1).** Encrypting a hundred-MB PDF must not buffer it in memory.
4. **Portability.** A project may travel between machines, mail systems, and zip viewers that
   mangle raw binary. The ciphertext should survive text-oriented transports.
5. **No hand-rolled crypto.** Repo security policy: never invent crypto; use a vetted library.

## Options considered

| Option | Passphrase UX | Per-entry | Streaming | Portable encoding | Library maturity |
| ------ | ------------- | --------- | --------- | ----------------- | ---------------- |
| Zip's built-in ZipCrypto | passphrase | per-entry | yes | n/a (in-zip) | **broken** — ZipCrypto is cryptographically weak. |
| Zip AES (WinZip AE-2) | passphrase | per-entry | yes | n/a (in-zip) | OK, but couples crypto to the zip layer and ties us to one zip crate's AES feature + C deps. |
| `age` with **passphrase (scrypt) recipient** | passphrase | per-entry (we wrap each entry) | **yes — chunked `StreamWriter`/`StreamReader`** | **yes — ASCII armor** | mature (str4d/rage), audited format. Chosen. |
| `age` with X25519 key recipients | key files (bad UX) | per-entry | yes | yes | rejected — violates constraint 1. |
| libsodium secretstream | passphrase (+ KDF we wire) | per-entry | yes | no (raw binary) | more wiring, less standard than age. |
| GPG symmetric | passphrase | per-entry | yes | yes (armor) | heavy dependency, awkward Rust story. |

## Decision

<Callout type="info" title="Decision">
  Encrypt with the **`age`** crate (`str4d/rage`) using a **passphrase (scrypt) recipient** and
  **ASCII armor** on every encrypted entry, leaving `manifest.json` in plaintext.
</Callout>

Concretely:

- **What is encrypted:** every zip entry *except* `manifest.json`. `manifest.encrypted = true`
  records the choice so a reader knows a passphrase is required for content (but not for the
  preview).
- **Passphrase handling:** the passphrase is carried as `age::secrecy::SecretString` (the
  zeroizing wrapper age re-exports — we do not add a separate `secrecy` dependency, keeping the
  version in lockstep). Encryption uses `age::Encryptor::with_user_passphrase`; decryption uses
  `age::scrypt::Identity`.
- **ASCII armor always on** (master plan §6.3). Encrypted entries are written through
  `age::armor::ArmoredWriter` (`Format::AsciiArmor`) and read through `ArmoredReader`. The cost
  is a ~33 % size increase from base64; the benefit is that ciphertext survives text-oriented
  transports and is recognizable (`-----BEGIN AGE ENCRYPTED FILE-----`). Because armored
  ciphertext is high-entropy base64, encrypted entries are `Stored` (not DEFLATEd) in the zip.
- **Streaming (Rule 1):** the write path pipes plaintext → `Encryptor` `StreamWriter` →
  `ArmoredWriter` → the current zip entry, and the read path reverses it, both via `io::copy`.
  No entry is buffered whole. The age stream is closed inside-out — finish the `StreamWriter`
  (flush the final ciphertext chunk), then finish the `ArmoredWriter` (emit the armor footer);
  skipping the second finish silently truncates the armor.
- **Wrong-passphrase is a typed error.** A wrong passphrase fails at decrypt time. age 0.11
  surfaces this as `DecryptError::DecryptionFailed` for a scrypt file (verified against the
  crate — *not* `NoMatchingKeys`, which the initial research assumed; with scrypt the passphrase
  is the only key, so an unwrap failure is a decryption failure). The crate maps
  `DecryptionFailed`, `KeyDecryptionFailed`, and `NoMatchingKeys` all to a single fieldless
  `FileFormatError::BadPassphrase` (stable code `fileformat.bad_passphrase`). The variant is
  fieldless on purpose so a wrong-passphrase attempt cannot leak age internals into a log or a
  user-facing message.

## Consequences

- `crates/shalgalt-fileformat/src/crypto.rs` is the **only** module that touches the `age` API.
  The rest of the crate stays crypto-library-agnostic; a future algorithm change is confined to
  one file.
- The `age` dependency is added with the non-default `armor` feature. Its transitive footprint
  (x25519-dalek, scrypt, etc.) is accepted as the cost of not hand-rolling crypto.
- `scrypt` work factor is auto-calibrated by age to ~1 second on the encrypting device. A file
  produced on a fast machine and opened on a slow one could trip age's `ExcessiveWork` guard;
  the crate maps that to a sanitized internal `Crypto` variant rather than `BadPassphrase`, so it
  is never mislabeled as a wrong passphrase. At the stable-code level `Crypto` folds into
  `fileformat.malformed` (the locked code set stays five values), so the desktop layer cannot
  today distinguish "device too slow" from "corrupt archive" by code alone — both surface as
  `fileformat.malformed`. If that UX distinction is ever needed, a sixth stable code can be added
  in a follow-up (and recorded here).
- **Passphrase recovery is impossible by design.** There is no escrow, no recovery key, no
  backdoor. A lost passphrase means a lost project. The optional `manifest.hint` field exists to
  reduce that risk; it MUST NOT contain the passphrase itself. The desktop UI (P4-03/04) must
  warn the teacher of this at the point they set a passphrase.
- No passphrase is ever persisted by this crate. The desktop layer decides whether to hold it in
  memory for a session; it is never written to disk or the SQLite DB.

## Follow-up

- P4-03 surfaces the passphrase prompt and the "passphrase cannot be recovered" warning in the
  Tauri command layer + frontend.
- The "encrypted entry's on-disk bytes never contain a plaintext substring of the payload"
  assertion (originally slated for P8 hardening) is already covered by
  `tests/roundtrip.rs::encrypted_archive_contains_no_payload_plaintext`, which guards against an
  accidental bypass of the encrypt path.

<Cards>
  <Card href="/adr/0011-shalgalt-file-format" title="ADR 0011 — `.shalgalt` project file">
    The zip container this encryption layer wraps, entry by entry.
  </Card>
</Cards>
