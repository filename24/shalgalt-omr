/**
 * `tauri-plugin-sql` 단일 진입점.
 *
 * 모든 DB 접근 모듈(`templates.ts`, `results.ts`, `students.ts`)은 `getDb()` 만 사용하고
 * 직접 `Database.load()` 를 호출하지 않는다. 이렇게 하면:
 *  - connection 식별자(`sqlite:shalgalt-omr.sqlite`)가 한 곳에 고정되고,
 *  - 첫 호출 시 1회만 로드되어 이후 호출은 캐시된 인스턴스를 재사용한다.
 *
 * 마이그레이션은 Rust 측 `tauri_plugin_sql::Builder::add_migrations` 로 등록되어 있으므로
 * 첫 `Database.load()` 시점에 자동 적용된다.
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
