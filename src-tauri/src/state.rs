//! Application state shared by every `tauri::command` handler and by the axum server.
//!
//! Rule 4: the axum server lives independently of the Tauri window, so it receives the
//! same `AppState` clone and never reaches into Tauri APIs.
//!
//! Database access is performed by the Tauri SQL plugin from the frontend, so this state
//! does NOT carry a database pool. Only OS-level resources (paths) are kept here.

use std::sync::Arc;

use crate::paths::AppDirs;

/// Cheap to clone — the underlying `Inner` is shared via `Arc`.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    pub dirs: AppDirs,
}

impl AppState {
    pub fn new(dirs: AppDirs) -> Self {
        Self {
            inner: Arc::new(Inner { dirs }),
        }
    }

    pub fn dirs(&self) -> &AppDirs {
        &self.inner.dirs
    }
}
