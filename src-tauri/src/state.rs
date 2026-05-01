//! 모든 `tauri::command`/`axum` 핸들러가 공유하는 애플리케이션 상태.
//!
//! Rule 4: axum 서버는 Tauri 윈도우와 무관하게 동일한 `AppState`만 받아 동작한다.
//!
//! DB 접근은 Tauri SQL 플러그인을 통해 프론트엔드에서 직접 수행하므로 본 상태에는
//! 데이터베이스 풀이 들어있지 않다. CV 임시 산출물 경로 등 OS 자원만 보관한다.

use std::sync::Arc;

use crate::paths::AppDirs;

/// `Clone`은 `Arc<Inner>`를 통해 저렴하게 수행한다.
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
