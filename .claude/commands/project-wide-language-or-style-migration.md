---
name: project-wide-language-or-style-migration
description: Workflow command scaffold for project-wide-language-or-style-migration in shalgalt-omr.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /project-wide-language-or-style-migration

Use this workflow when working on **project-wide-language-or-style-migration** in `shalgalt-omr`.

## Goal

Migrate codebase, documentation, and UI strings to a consistent language or style.

## Common Files

- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/BLUEPRINT.md`
- `src-tauri/Cargo.toml`
- `src-tauri/src/**/*.rs`
- `src/lib/**/*.ts`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Edit multiple files across backend, frontend, and docs to update language or style
- Commit with a chore: or docs: prefix in the message

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.