---
name: add-or-evolve-pdf-feature
description: Workflow command scaffold for add-or-evolve-pdf-feature in shalgalt-omr.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /add-or-evolve-pdf-feature

Use this workflow when working on **add-or-evolve-pdf-feature** in `shalgalt-omr`.

## Goal

Adds a new PDF-related feature or evolves the PDF generation capabilities, including Rust crate changes, asset additions, and TypeScript type updates.

## Common Files

- `crates/shalgalt-pdf/src/*.rs`
- `crates/shalgalt-pdf/assets/fonts/*`
- `crates/shalgalt-pdf/tests/*.rs`
- `crates/shalgalt-core/src/domain/*.rs`
- `Cargo.lock`
- `crates/shalgalt-pdf/Cargo.toml`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Update or create files in crates/shalgalt-pdf/src/ (e.g., new modules, layout, style, shapes)
- Add or update font assets in crates/shalgalt-pdf/assets/fonts/
- Update or create tests in crates/shalgalt-pdf/tests/ (including golden tests and smoke tests)
- Update or create related domain files in crates/shalgalt-core/src/domain/
- Update Cargo.lock and relevant Cargo.toml files

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.