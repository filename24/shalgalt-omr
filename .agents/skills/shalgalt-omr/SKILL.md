```markdown
# shalgalt-omr Development Patterns

> Auto-generated skill from repository analysis

## Overview

This skill teaches you how to contribute effectively to the `shalgalt-omr` project, a TypeScript codebase built with Vite and Svelte. The repository features a hybrid Rust/TypeScript backend (via Tauri), a modular Svelte-based UI, and workflows for database migrations, feature development, and internationalization. You'll learn the project's coding conventions, how to implement common workflows, and how to write and organize tests.

## Coding Conventions

- **File Naming:** Use `camelCase` for file and directory names.
  - Example: `templateEditor.svelte`, `dbAccessors.ts`
- **Import Style:** Use relative imports.
  - Example:
    ```ts
    import { fetchTemplates } from './dbAccessors';
    ```
- **Export Style:** Use named exports.
  - Example:
    ```ts
    // dbAccessors.ts
    export function fetchTemplates() { ... }
    ```
- **Commit Messages:** Freeform, often with `p0` or `p1` prefixes.
  - Example: `p1 add support for question image upload`

## Workflows

### Add or Modify Database Table or Schema

**Trigger:** When you need to add a new table or make a schema change in the database.  
**Command:** `/new-table`

1. **Create or modify a migration file**  
   Add a new `.sql` file in `src-tauri/migrations/` for your migration.
   ```sql
   -- src-tauri/migrations/20240601_add_scores_table.sql
   CREATE TABLE scores (
     id INTEGER PRIMARY KEY,
     user_id INTEGER,
     score INTEGER
   );
   ```
2. **Update Rust domain model**  
   Edit or add the relevant struct in `src-tauri/src/domain/`.
   ```rust
   // src-tauri/src/domain/score.rs
   pub struct Score {
       pub id: i32,
       pub user_id: i32,
       pub score: i32,
   }
   ```
3. **Update or create TypeScript DB accessors**  
   Add or update functions in `src/lib/db/`.
   ```ts
   // src/lib/db/scores.ts
   export function getScoresByUser(userId: number) { ... }
   ```
4. **Update TypeScript types**  
   Edit or add types in `src/lib/types/`.
   ```ts
   // src/lib/types/score.ts
   export type Score = {
     id: number;
     userId: number;
     score: number;
   };
   ```
5. **Update or add fixtures/tests**  
   Add or update tests in `src-tauri/tests/` to cover the new schema.

---

### Feature Development with Editor Components

**Trigger:** When adding a new editor feature or UI capability.  
**Command:** `/new-editor-feature`

1. **Create new Svelte components**  
   Place new components in `src/lib/components/editor/`.
   ```svelte
   <!-- src/lib/components/editor/scoreInput.svelte -->
   <script lang="ts">
     export let score: number;
   </script>
   <input type="number" bind:value={score} />
   ```
2. **Add supporting TypeScript logic**  
   Implement helper functions in the same directory.
   ```ts
   // src/lib/components/editor/scoreHelpers.ts
   export function validateScore(score: number): boolean { ... }
   ```
3. **Update or add Svelte stores**  
   Add or update stores in `src/lib/stores/`.
   ```ts
   // src/lib/stores/scoreStore.ts
   import { writable } from 'svelte/store';
   export const scores = writable<number[]>([]);
   ```
4. **Update or add DB accessors**  
   If needed, modify `src/lib/db/`.
5. **Update or add IPC handlers**  
   If backend communication is needed, update `src/lib/ipc/`.
6. **Wire up new components**  
   Integrate your component in `src/routes/` or existing shells.

---

### Add or Update i18n and UI Strings

**Trigger:** When adding new UI strings or supporting a new language.  
**Command:** `/add-i18n-string`

1. **Update or add i18n files**  
   Edit or create entries in `src/lib/i18n/`.
   ```ts
   // src/lib/i18n/en.ts
   export default {
     scoreLabel: "Score",
   };
   ```
2. **Update Svelte components to use new i18n keys**  
   ```svelte
   <script lang="ts">
     import i18n from '../../i18n';
   </script>
   <label>{i18n.scoreLabel}</label>
   ```
3. **Update documentation if language conventions change**  
   Edit `AGENTS.md` or related docs as needed.

## Testing Patterns

- **Framework:** [vitest](https://vitest.dev/)
- **Test File Pattern:** Place tests alongside code or in a `tests/` directory, using the `*.test.ts` suffix.
  - Example: `src/lib/db/scores.test.ts`
- **Example Test:**
  ```ts
  import { getScoresByUser } from './scores';

  test('fetches scores for a user', () => {
    expect(getScoresByUser(1)).toEqual([/* expected scores */]);
  });
  ```

## Commands

| Command              | Purpose                                                        |
|----------------------|----------------------------------------------------------------|
| /new-table           | Start a database table or schema change workflow               |
| /new-editor-feature  | Begin developing a new editor feature/component                |
| /add-i18n-string     | Add or update i18n strings and UI translations                 |
```
