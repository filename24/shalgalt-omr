---
name: add-database-table-or-schema-change
description: Workflow command scaffold for add-database-table-or-schema-change in shalgalt-omr.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /add-database-table-or-schema-change

Use this workflow when working on **add-database-table-or-schema-change** in `shalgalt-omr`.

## Goal

Adds or modifies a database table or schema, including migrations and updating related TypeScript types and accessors.

## Common Files

- `src-tauri/migrations/*.sql`
- `src-tauri/src/domain/*.rs`
- `src/lib/db/*.ts`
- `src/lib/types/*.ts`
- `src-tauri/tests/*`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Create or modify a migration file in src-tauri/migrations/*.sql
- Update Rust domain model in src-tauri/src/domain/*.rs
- Update or create TypeScript DB accessors in src/lib/db/*.ts
- Update TypeScript types in src/lib/types/*.ts
- Update or add fixtures/tests in src-tauri/tests/*

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.