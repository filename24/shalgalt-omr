---
name: feature-development-with-editor-components
description: Workflow command scaffold for feature-development-with-editor-components in shalgalt-omr.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /feature-development-with-editor-components

Use this workflow when working on **feature-development-with-editor-components** in `shalgalt-omr`.

## Goal

Implements a new feature, especially for the template editor, by creating new Svelte components, updating stores, and wiring up IPC and DB access.

## Common Files

- `src/lib/components/editor/*.svelte`
- `src/lib/components/editor/*.ts`
- `src/lib/stores/*.ts`
- `src/lib/db/*.ts`
- `src/lib/ipc/*.ts`
- `src/routes/*`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Create new Svelte components in src/lib/components/editor/
- Update or add supporting TypeScript logic in src/lib/components/editor/*.ts
- Update or add Svelte stores in src/lib/stores/*.ts
- Update or add DB accessors in src/lib/db/*.ts if needed
- Update or add IPC handlers in src/lib/ipc/*.ts

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.