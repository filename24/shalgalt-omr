//! Tauri 앱 진입점.
//!
//! 부팅 순서 (Blueprint Rule 4 + ARCHITECTURE §4):
//!   1. tracing 초기화
//!   2. AppDirs 해석 → 데이터/스캔 디렉터리 보장
//!   3. Tauri SQL 플러그인 등록 (마이그레이션 포함) — DB CRUD는 프론트엔드 담당
//!   4. AppState 조립
//!   5. axum 백그라운드 서버 spawn (Rule 4)
//!   6. tauri::Builder 부착 + invoke_handler 등록

mod api;
mod commands;
mod domain;
mod error;
mod export;
mod grading;
mod paths;
mod scan;
mod state;

use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use crate::api::ApiHandle;
use crate::paths::{AppDirs, DB_FILENAME};
use crate::state::AppState;

/// 프론트엔드와 동일한 connection 식별자. AppConfig 디렉터리 기준 상대경로.
const DB_URL: &str = "sqlite:shalgalt-omr.sqlite";

/// 마이그레이션 SQL은 별도 파일에 보관하고 컴파일 타임에 임베드.
const MIGRATION_0001: &str = include_str!("../migrations/0001_init.sql");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let migrations = vec![Migration {
        version: 1,
        description: "init_students_templates_results",
        sql: MIGRATION_0001,
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(DB_URL, migrations)
                .build(),
        )
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = bootstrap(handle).await {
                    tracing::error!("bootstrap failed: {e:?}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan::scan_grade_pdf,
            commands::export::export_results_xlsx,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_tracing() {
    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .try_init();
}

async fn bootstrap(app: tauri::AppHandle) -> anyhow::Result<()> {
    let dirs = AppDirs::resolve()?;
    info!(
        "data dir: {}, db file: {}",
        dirs.data_dir.display(),
        DB_FILENAME
    );

    let state = AppState::new(dirs);

    // Rule 4: axum 서버는 별도 task. 핸들은 AppHandle::manage로 보관해 종료 시 drop.
    let api: ApiHandle = api::spawn(state.clone()).await?;
    app.manage(api);
    app.manage(state);

    Ok(())
}
