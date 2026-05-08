//! Tauri application entry point.
//!
//! Boot order (Blueprint Rule 4 + ARCHITECTURE §4):
//!   1. initialize tracing
//!   2. resolve `AppDirs` and ensure data/scan directories exist
//!   3. register single-instance / window-state / log plugins (desktop QoL)
//!   4. register the Tauri SQL plugin (with embedded migrations) — DB CRUD lives in the UI
//!   5. assemble `AppState`
//!   6. spawn the axum background server (Rule 4)
//!   7. attach `tauri::Builder` and register the invoke handler

// P0 scaffolding exposes APIs that are wired up in P1–P4. Remove these allows once the
// CV pipeline (P2) and grading engine (P3) consume the stubbed accessors and re-exports.
#![allow(dead_code, unused_imports)]

// Domain / grading / export / api router / error envelope live in
// `shalgalt-core` (P2-02). The CV pipeline (`scan`/`preview`/`pipeline`) lives
// in `shalgalt-cv` (P2-03). Only Tauri-bound modules stay in this crate.
mod api;
mod commands;
mod paths;
mod state;

use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use crate::api::ApiHandle;
use crate::paths::{AppDirs, DB_FILENAME};
use crate::state::AppState;

/// Connection identifier shared with the frontend. Resolved relative to AppConfig.
const DB_URL: &str = "sqlite:shalgalt-omr.sqlite";

/// Migration SQL is kept in a sibling file and embedded at compile time.
const MIGRATION_0001: &str = include_str!("../migrations/0001_init.sql");
const MIGRATION_0002: &str = include_str!("../migrations/0002_backdrop.sql");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let migrations = vec![
        Migration {
            version: 1,
            description: "init_students_templates_results",
            sql: MIGRATION_0001,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add_templates_backdrop_path",
            sql: MIGRATION_0002,
            kind: MigrationKind::Up,
        },
    ];

    let mut builder = tauri::Builder::default();

    // Single-instance MUST be the first plugin registered. When a second copy launches
    // it forwards argv/cwd to the already-running app, which focuses its main window.
    // Critical here because two processes on the same SQLite file can corrupt state.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }));
    }

    // Restore the last window size/position automatically.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());
    }

    builder
        .plugin(
            // Persistent rotating log file in the OS log dir, plus stdout for `tauri dev`
            // and the webview console for frontend `info!` / `warn!` calls.
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("shalgalt-omr".into()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
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
            commands::scan::rasterize_pdf_first_page,
            commands::export::export_results_xlsx,
            commands::pdf::pdf_generate_omr,
            commands::pdf::pdf_render_template_preview,
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
    let dirs = AppDirs::resolve(&app)?;
    info!(
        "data dir: {}, db file: {}",
        dirs.data_dir.display(),
        DB_FILENAME
    );

    // Drop preview cache files older than 24h before the editor opens.
    commands::pdf::prune_old_previews(&dirs.cache_dir);

    let state = AppState::new(dirs);

    // Rule 4: the axum server lives in its own task. Storing the handle via `manage` means
    // it is dropped (and shut down gracefully) when the Tauri app exits.
    let api: ApiHandle = api::spawn().await?;
    app.manage(api);
    app.manage(state);

    Ok(())
}
