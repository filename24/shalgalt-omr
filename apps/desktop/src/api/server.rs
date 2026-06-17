//! Spawn the axum server on a separate tokio task and return a graceful-shutdown handle.
//!
//! Rule 4: it must live independently of the Tauri main-window lifecycle, so a oneshot
//! channel is used as the shutdown signal. The router itself comes from `shalgalt-core`
//! so `apps/server` (P5-05) can reuse it without pulling in any Tauri code.

use std::net::SocketAddr;
use std::sync::Arc;

use shalgalt_core::api::{cors::permissive, router, AppState, MemoryStore};
use shalgalt_core::error::{AppError, AppResult};
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
/// The router is fed a read-only `DataStore`. P5 wires this to a `rusqlite` reader over the
/// plugin-sql SQLite file; until that lands it serves an empty `MemoryStore` so the
/// `/healthz` and `/v1/` surface is reachable without a database.
pub async fn spawn() -> AppResult<ApiHandle> {
    let addr: SocketAddr = "127.0.0.1:8080".parse().expect("hardcoded addr");
    let state = AppState::new(Arc::new(MemoryStore::read_only()));
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
