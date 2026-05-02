```markdown
# shalgalt-omr Development Patterns

> Auto-generated skill from repository analysis

## Overview
This skill covers the development patterns and workflows for the `shalgalt-omr` project—a Rust backend with a Vite-powered frontend. The repository emphasizes clear documentation, consistent code style, and structured commit messages. You'll learn how to contribute code, update documentation, and maintain consistency across the codebase.

## Coding Conventions

### File Naming
- Use **camelCase** for file names.
  - Example: `scanSheet.rs`, `answerParser.ts`, `userProfile.svelte`

### Import Style
- Mixed import styles are used. Both default and named imports may appear.
  - Example (TypeScript/Svelte):
    ```ts
    import { parseAnswers } from './answerParser';
    import scanSheet from './scanSheet';
    ```

### Export Style
- Prefer **named exports**.
  - Example (TypeScript):
    ```ts
    export function parseAnswers(data: string) { ... }
    export const SHEET_VERSION = '1.0.0';
    ```

### Commit Messages
- Follow **conventional commit** style.
- Prefixes: `docs:`, `feat:`, `chore:`
- Example:
  ```
  docs: update README with new installation instructions
  feat: add answer sheet parsing logic
  chore: migrate UI strings to English
  ```

## Workflows

### Documentation Update
**Trigger:** When you want to add new documentation or update existing docs.  
**Command:** `/update-docs`

1. Edit or create markdown files in the `docs/` directory or the root `README.md`.
2. Commit your changes with a `docs:` prefix in the commit message.
   - Example: `docs: add usage guide for answer scanning`
3. Submit your pull request for review.

**Files Involved:**
- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/BLUEPRINT.md`
- `AGENTS.md`

---

### Project-wide Language or Style Migration
**Trigger:** When you need to enforce language or style conventions across the project.  
**Command:** `/migrate-language`

1. Edit multiple files across backend, frontend, and documentation to update language or style.
   - This may include renaming variables, updating UI strings, or standardizing documentation language.
2. Commit your changes with a `chore:` or `docs:` prefix in the commit message.
   - Example: `chore: migrate all UI text to English`
3. Submit your pull request for review.

**Files Involved:**
- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/BLUEPRINT.md`
- `src-tauri/Cargo.toml`
- `src-tauri/src/**/*.rs`
- `src/lib/**/*.ts`
- `src/lib/**/*.svelte`
- `src/routes/**/*.svelte`

---

## Testing Patterns

- **Test File Pattern:** Files ending with `.test.*` (e.g., `answerParser.test.ts`)
- **Testing Framework:** Not explicitly detected; check existing test files for framework clues.
- **Location:** Tests are typically placed alongside the code they test.

**Example Test File:**
```ts
// answerParser.test.ts
import { parseAnswers } from './answerParser';

test('parses valid answer sheet', () => {
  const input = 'A,B,C,D';
  expect(parseAnswers(input)).toEqual(['A', 'B', 'C', 'D']);
});
```

## Commands

| Command         | Purpose                                                |
|-----------------|--------------------------------------------------------|
| /update-docs    | Add or update project documentation and guides         |
| /migrate-language | Migrate codebase, docs, and UI strings for consistency |

```