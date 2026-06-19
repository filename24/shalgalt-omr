```markdown
# shalgalt-omr Development Patterns

> Auto-generated skill from repository analysis

## Overview

This skill provides guidance on contributing to the `shalgalt-omr` codebase, a Rust project focused on Optical Mark Recognition (OMR) with PDF generation capabilities. It covers coding conventions, commit patterns, and step-by-step workflows for evolving PDF features, ensuring consistency and maintainability across the repository.

## Coding Conventions

### File Naming

- Use **camelCase** for file names.
  - Example: `pdfLayout.rs`, `fontManager.rs`

### Import Style

- Use **relative imports** within modules.
  - Example (Rust):
    ```rust
    mod layout;
    use crate::layout::PageLayout;
    ```
  - Example (TypeScript):
    ```typescript
    import { GeneratedType } from './generated/typeDefs';
    ```

### Export Style

- Use **named exports**.
  - Example (TypeScript):
    ```typescript
    export type { GeneratedType };
    ```

### Commit Messages

- Follow **conventional commit** style.
- Prefix with `feat` for new features.
  - Example:
    ```
    feat(pdf): add support for custom font embedding in PDF output
    ```

## Workflows

### Add or Evolve PDF Feature

**Trigger:** When introducing or significantly updating PDF generation/layout features.

**Command:** `/add-pdf-feature`

Follow these steps to add or evolve PDF-related features:

1. **Update or create Rust modules** in `crates/shalgalt-pdf/src/`.
   - Add new files for layouts, styles, shapes, etc.
   - Example:
     ```rust
     // crates/shalgalt-pdf/src/layout.rs
     pub struct PageLayout { /* ... */ }
     ```
2. **Add or update font assets** in `crates/shalgalt-pdf/assets/fonts/`.
   - Place new `.ttf` or `.otf` files as needed.
3. **Update or create tests** in `crates/shalgalt-pdf/tests/`.
   - Include both golden tests (output comparison) and smoke tests.
   - Example:
     ```rust
     #[test]
     fn test_pdf_layout() {
         // test implementation
     }
     ```
4. **Update domain logic** in `crates/shalgalt-core/src/domain/` as necessary.
   - Add or modify domain models/types related to PDF features.
5. **Update Cargo files**:
   - Run `cargo build` to update `Cargo.lock`.
   - Edit `crates/shalgalt-pdf/Cargo.toml` if dependencies change.
6. **Update generated TypeScript types** in `src/lib/types/generated/`.
   - Regenerate or manually update `.ts` files to reflect Rust changes.
   - Example:
     ```typescript
     // src/lib/types/generated/pdfTypes.ts
     export type PdfLayout = { /* ... */ };
     ```
7. **(Optional) Add documentation** in `docs/adr/`.
   - Create or update architecture decision records for major changes.

## Testing Patterns

- **Framework:** Unknown (Rust's built-in test framework is likely).
- **Test Files:** Use `*.test.ts` for TypeScript tests; Rust tests are in `crates/shalgalt-pdf/tests/*.rs`.
- **Test Example (Rust):**
  ```rust
  #[test]
  fn test_pdf_generation() {
      // Arrange, Act, Assert
  }
  ```
- **Test Example (TypeScript):**
  ```typescript
  // src/lib/types/generated/pdfTypes.test.ts
  import { PdfLayout } from './pdfTypes';

  test('PdfLayout structure', () => {
    // test implementation
  });
  ```

## Commands

| Command           | Purpose                                                      |
|-------------------|--------------------------------------------------------------|
| /add-pdf-feature  | Initiate the workflow for adding or evolving PDF features    |
```
