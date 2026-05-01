//! Tauri IPC 진입점 모음.
//!
//! 각 핸들러는 검증 → 도메인/CV 호출 → `AppError` 변환만 수행한다 (박막 원칙).
//!
//! DB CRUD 는 `tauri-plugin-sql`을 통해 프론트엔드에서 직접 수행하므로
//! 이 모듈에는 데이터 read/write 명령이 없다.
//! 여기에 남는 것은 (a) Rust 전용 무거운 작업(스캔), (b) 외부 포맷 변환(xlsx) 뿐이다.

pub mod export;
pub mod scan;
