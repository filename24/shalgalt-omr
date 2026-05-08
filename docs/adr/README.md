# Architecture Decision Records

This folder collects ADRs — short documents that record significant architectural or
operational choices, the alternatives considered, and the consequences accepted.

## When to write one

- The decision is hard to reverse (e.g. choice of build toolchain, database engine,
  IPC contract).
- Multiple credible options exist and the trade-offs are non-obvious.
- A future contributor would otherwise re-litigate the same question.

If a change is small, local, and easily reversible, it does not need an ADR — just write
the code.

## File naming

`NNNN-kebab-case-title.md`, monotonically increasing (no gaps). Use four digits to keep
sort order stable past 100 entries.

## Template

Use the structure of an existing ADR (Status / Date / Deciders / Context / Options /
Decision / Consequences / Follow-up). Keep them short — the value is in the *recorded
reasoning*, not the prose volume.

## Index

| ID | Title | Status |
| -- | ----- | ------ |
| [0001](0001-windows-opencv-strategy.md) | Windows OpenCV install strategy | Accepted |
| [0002](0002-pdf-generator-printpdf.md) | PDF generator: `printpdf` | Accepted |
| [0003](0003-cargo-workspace-and-crate-boundaries.md) | Cargo workspace + crate boundaries | Accepted |
| [0004](0004-ts-rs-type-codegen.md) | Generate TypeScript bindings from Rust via `ts-rs` | Accepted |
| [0005](0005-pdf-ipc-contract.md) | PDF generation IPC contract | Accepted |
| [0006](0006-light-theme-and-comfort-typography.md) | Light-default theme + comfortable typography mode | Accepted |
