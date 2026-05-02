---
name: documentation-update
description: Workflow command scaffold for documentation-update in shalgalt-omr.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /documentation-update

Use this workflow when working on **documentation-update** in `shalgalt-omr`.

## Goal

Add or update project documentation and guides.

## Common Files

- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/BLUEPRINT.md`
- `AGENTS.md`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Edit or create markdown files in the docs/ directory or root README.md
- Commit with a docs: prefix in the message

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.