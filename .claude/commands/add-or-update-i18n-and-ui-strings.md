---
name: add-or-update-i18n-and-ui-strings
description: Workflow command scaffold for add-or-update-i18n-and-ui-strings in shalgalt-omr.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /add-or-update-i18n-and-ui-strings

Use this workflow when working on **add-or-update-i18n-and-ui-strings** in `shalgalt-omr`.

## Goal

Adds or updates internationalization (i18n) files and ensures UI strings are in the correct language.

## Common Files

- `src/lib/i18n/*.ts`
- `src/lib/components/**/*.svelte`
- `AGENTS.md`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Update or add i18n files in src/lib/i18n/*.ts
- Update Svelte components to use new i18n keys
- Update documentation if language conventions change

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.