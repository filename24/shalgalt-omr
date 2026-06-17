//! Spawn the axum server on a separate tokio task and return a graceful-shutdown handle.
//!
//! Rule 4: it must live independently of the Tauri main-window lifecycle, so a oneshot
//! channel is used as the shutdown signal. The router itself comes from `shalgalt-core`
//! so `apps/server` (P5-05) can reuse it without pulling in any Tauri code.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use shalgalt_core::api::{cors::permissive, router, AppState};
use shalgalt_core::error::{AppError, AppResult};
use shalgalt_store::DeferredReadOnlyStore;
use tokio::sync::oneshot;
use tracing::{error, info};

/// Token used to terminate the server. Dropping it triggers shutdown automatically.
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

/// Bind the axum server on `127.0.0.1:8080` (Blueprint §2).
///
/// `db_path` is the plugin-sql SQLite file. The router is fed a [`DeferredReadOnlyStore`]
/// over it: the desktop's HTTP surface reads real exam/template/result data but never
/// writes (plugin-sql owns writes — Rule: no DB writes in Rust). The store degrades to
/// empty reads until plugin-sql lazily creates the file on the frontend's first query.
pub async fn spawn(db_path: PathBuf) -> AppResult<ApiHandle> {
    let addr: SocketAddr = "127.0.0.1:8080".parse().expect("hardcoded addr");
    let state = AppState::new(Arc::new(DeferredReadOnlyStore::new(db_path)));
    let app = router(state).layer(permissive());

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
