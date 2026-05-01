//! axum 서버를 별도 tokio task로 띄우고, graceful shutdown 핸들을 반환한다.
//!
//! Rule 4: Tauri 메인 윈도우 라이프사이클과 무관해야 하므로 별도 oneshot 시그널로 종료한다.

use std::net::SocketAddr;

use tokio::sync::oneshot;
use tracing::{error, info};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// 서버를 종료하는 토큰. `Drop` 시 자동 셧다운.
pub struct ApiHandle {
    shutdown: Option<oneshot::Sender<()>>,
}

impl ApiHandle {
    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for ApiHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// `127.0.0.1:8080` 에서 axum 서버를 띄운다 (Blueprint §2).
pub async fn spawn(state: AppState) -> AppResult<ApiHandle> {
    let addr: SocketAddr = "127.0.0.1:8080".parse().expect("hardcoded addr");
    let app = super::routes::router(state).layer(super::cors::permissive());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("bind {addr}: {e}")))?;

    let (tx, rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        info!("axum api server listening on http://{addr}");
        let result = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await;
        if let Err(e) = result {
            error!("axum server stopped with error: {e}");
        }
    });

    Ok(ApiHandle { shutdown: Some(tx) })
}
