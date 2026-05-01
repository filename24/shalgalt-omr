/**
 * Tauri IPC wrapper barrel.
 *
 * 여기에는 **Rust 전용 명령**(스캔, 엑셀 내보내기 등)만 모인다.
 * DB CRUD 는 `tauri-plugin-sql` 을 통해 직접 처리하므로 `$lib/db/*` 모듈을 사용한다.
 */
export * from "./scan";
export * from "./export";
