/**
 * Tauri IPC wrapper barrel.
 *
 * Only **Rust-only commands** (scan, xlsx export, ...) live here. DB CRUD goes through
 * `tauri-plugin-sql` and lives in `$lib/db/*`.
 */
export * from "./scan";
export * from "./export";
export * from "./pdf";
export * from "./project";
