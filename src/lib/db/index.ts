/**
 * Single entry point for `tauri-plugin-sql`.
 *
 * Every DB module (`templates.ts`, `results.ts`, `students.ts`) goes through `getDb()` and
 * never calls `Database.load()` directly. This way:
 *  - the connection identifier (`sqlite:shalgalt-omr.sqlite`) lives in one place, and
 *  - the first call performs the load once; subsequent calls reuse the cached instance.
 *
 * Migrations are registered via `tauri_plugin_sql::Builder::add_migrations` on the Rust
 * side, so they are applied automatically on the very first `Database.load()`.
 */
import Database from "@tauri-apps/plugin-sql";

export const DB_URL = "sqlite:shalgalt-omr.sqlite";

let dbPromise: Promise<Database> | null = null;

export function getDb(): Promise<Database> {
  if (!dbPromise) {
    dbPromise = Database.load(DB_URL);
  }
  return dbPromise;
}
