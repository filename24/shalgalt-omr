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

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{Emitter, Manager};
use tauri_plugin_sql::{Migration, MigrationKind};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use crate::api::ApiHandle;
use crate::paths::{AppDirs, DB_FILENAME};
use crate::state::AppState;

/// Connection identifier shared with the frontend. Resolved relative to AppConfig.
const DB_URL: &str = "sqlite:shalgalt-omr.sqlite";

/// Event name carrying a `.shalgalt` path the OS asked us to open (P4-07).
const OPEN_WITH_FILE_EVENT: &str = "open-with-file";

/// A `.shalgalt` path the OS handed us that the webview has not consumed yet.
///
/// Two producers feed it: the cold-start argv sniff (`first_shalgalt_arg`) and a
/// macOS `RunEvent::Opened` that may fire before the frontend attaches its
/// `open-with-file` listener. The frontend drains it once on startup via
/// [`take_pending_open_file`]; warm "open with" while running is delivered by the
/// event instead. (P4-07)
#[derive(Default)]
struct PendingOpenFile(Mutex<Option<PathBuf>>);

/// First CLI argument that looks like a `.shalgalt` file, if any. On Windows and
/// Linux the OS passes the double-clicked path as argv; macOS uses `RunEvent::Opened`.
fn first_shalgalt_arg<I>(args: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = String>,
{
    args.into_iter().skip(1).map(PathBuf::from).find(|p| {
        p.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("shalgalt"))
    })
}

/// Drain the pending "open with" path captured at cold start or by an early macOS
/// `Opened` event. The frontend calls this once on startup; returns `None` when the
/// app was launched normally. (P4-07)
#[tauri::command]
fn take_pending_open_file(state: tauri::State<'_, PendingOpenFile>) -> Option<String> {
    state
        .0
        .lock()
        .ok()
        .and_then(|mut guard| guard.take())
        .map(|p| p.to_string_lossy().into_owned())
}

/// Migration SQL is kept in a sibling file and embedded at compile time.
const MIGRATION_0001: &str = include_str!("../migrations/0001_init.sql");
const MIGRATION_0002: &str = include_str!("../migrations/0002_backdrop.sql");
const MIGRATION_0003: &str = include_str!("../migrations/0003_jobs.sql");
const MIGRATION_0004: &str = include_str!("../migrations/0004_exams.sql");
const MIGRATION_0005: &str = include_str!("../migrations/0005_results_exam.sql");

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
        Migration {
            version: 3,
            description: "add_jobs_table",
            sql: MIGRATION_0003,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "add_exams_answer_keys",
            sql: MIGRATION_0004,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "add_results_exam_id",
            sql: MIGRATION_0005,
            kind: MigrationKind::Up,
        },
    ];

    // Path the OS asked us to open at cold start (Windows/Linux argv). macOS delivers
    // it via `RunEvent::Opened` below instead.
    let initial_open = first_shalgalt_arg(std::env::args());

    let mut builder = tauri::Builder::default().manage(PendingOpenFile(Mutex::new(initial_open)));

    // Single-instance MUST be the first plugin registered. When a second copy launches
    // it forwards argv/cwd to the already-running app, which focuses its main window.
    // Critical here because two processes on the same SQLite file can corrupt state.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            // A second launch with a `.shalgalt` argument (double-click while running)
            // routes the file into the existing window (P4-07).
            if let Some(path) = first_shalgalt_arg(args) {
                if let Some(state) = app.try_state::<PendingOpenFile>() {
                    if let Ok(mut guard) = state.0.lock() {
                        *guard = Some(path.clone());
                    }
                }
                let _ = app.emit(OPEN_WITH_FILE_EVENT, path.to_string_lossy().into_owned());
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
            commands::scan::scan_answer_key,
            commands::scan::regrade_sheet,
            commands::scan::rasterize_pdf_first_page,
            commands::export::export_results_xlsx,
            commands::pdf::pdf_generate_omr,
            commands::pdf::pdf_render_template_preview,
            commands::project::project_open,
            commands::project::project_save,
            commands::project::project_export,
            take_pending_open_file,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {
            // macOS delivers "open with" / file-association double-clicks as an
            // `Opened` run event carrying file URLs (P4-07). Windows/Linux use argv,
            // handled by the cold-start sniff and the single-instance callback.
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            if let tauri::RunEvent::Opened { urls } = &_event {
                // Open a single project at a time: route the first resolvable file and
                // ignore the rest, so the pending slot is not overwritten by later URLs.
                if let Some(path) = urls.iter().find_map(|u| u.to_file_path().ok()) {
                    if let Some(state) = _app.try_state::<PendingOpenFile>() {
                        if let Ok(mut guard) = state.0.lock() {
                            *guard = Some(path.clone());
                        }
                    }
                    let _ = _app.emit(OPEN_WITH_FILE_EVENT, path.to_string_lossy().into_owned());
                }
            }
        });
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
