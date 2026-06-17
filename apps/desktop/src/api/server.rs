//! Spawn the axum server on a separate tokio task and return a graceful-shutdown handle.
//!
//! Rule 4: it must live independently of the Tauri main-window lifecycle, so a oneshot
//! channel is used as the shutdown signal. The router itself comes from `shalgalt-core`
//! so `apps/server` (P5-05) can reuse it without pulling in any Tauri code.
//!
//! Port hardening (ADR 0015): the embedded API binds a loopback port that defaults to
//! [`DEFAULT_API_PORT`] (away from the crowded 8080/8000/3000/5173 dev range). If that port
//! is busy on a teacher's machine the binder auto-falls-back to the next free port, and the
//! port actually bound is advertised through [`write_endpoint_file`] so external
//! integrations can still find it. A developer may pin the starting port with the
//! [`API_PORT_ENV`] environment variable; fallback still applies.

use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use shalgalt_core::api::{cors::permissive, router, AppState};
use shalgalt_core::error::{AppError, AppResult};
use shalgalt_store::DeferredReadOnlyStore;
use tokio::sync::oneshot;
use tracing::{error, info, warn};

/// Default loopback port for the desktop's embedded API. Chosen in the IANA *registered*
/// range (1024–49151, so the OS never hands it out as an ephemeral port) and deliberately
/// away from common dev defaults (8080/8000/3000/5173/…) to minimize collisions on a
/// teacher's machine. See ADR 0015.
pub const DEFAULT_API_PORT: u16 = 22345;

/// Environment variable a developer can set to pin the desired *starting* port. Auto-fallback
/// still applies if the chosen port is busy. See ADR 0015.
pub const API_PORT_ENV: &str = "SHALGALT_API_PORT";

/// File name (under the app data dir) where the embedded API advertises the port it bound,
/// so external integrations can discover it after auto-fallback. See ADR 0015.
pub const ENDPOINT_FILENAME: &str = "api-endpoint.json";

/// Number of consecutive ports to try (`desired`, `desired + 1`, …) before giving up.
const FALLBACK_ATTEMPTS: u16 = 16;

/// Token used to terminate the server. Dropping it triggers shutdown automatically.
pub struct ApiHandle {
    shutdown: Option<oneshot::Sender<()>>,
    /// The port the server actually bound — may differ from the desired port after fallback.
    port: u16,
}

impl ApiHandle {
    /// The loopback port the embedded API is serving on.
    pub fn port(&self) -> u16 {
        self.port
    }

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

/// Resolve the desired starting port from [`API_PORT_ENV`], falling back to
/// [`DEFAULT_API_PORT`] when unset or invalid.
fn desired_port() -> u16 {
    parse_desired_port(std::env::var(API_PORT_ENV).ok())
}

/// Pure helper for [`desired_port`] — kept env-free so it is unit-testable.
fn parse_desired_port(raw: Option<String>) -> u16 {
    match raw {
        Some(s) => match s.trim().parse::<u16>() {
            Ok(p) if p != 0 => p,
            _ => {
                warn!(
                    "ignoring invalid {API_PORT_ENV}={s:?}; using default port {DEFAULT_API_PORT}"
                );
                DEFAULT_API_PORT
            }
        },
        None => DEFAULT_API_PORT,
    }
}

/// Bind the first free port in `[desired, desired + FALLBACK_ATTEMPTS)`. Only `AddrInUse`
/// triggers fallback; any other bind error (e.g. permission) is returned immediately so we
/// do not mask a real misconfiguration behind a port walk.
async fn bind_with_fallback(
    host: Ipv4Addr,
    desired: u16,
) -> std::io::Result<tokio::net::TcpListener> {
    let mut last_err: Option<std::io::Error> = None;
    for offset in 0..FALLBACK_ATTEMPTS {
        let port = desired.saturating_add(offset);
        let addr = SocketAddr::from((host, port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                if offset > 0 {
                    warn!("desired API port {desired} was busy; bound fallback port {port}");
                }
                return Ok(listener);
            }
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                last_err = Some(e);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        std::io::Error::new(ErrorKind::AddrInUse, "no free port in fallback range")
    }))
}

/// Bind the axum server on a loopback port and serve it on a background task.
///
/// `db_path` is the plugin-sql SQLite file. The router is fed a [`DeferredReadOnlyStore`]
/// over it: the desktop's HTTP surface reads real exam/template/result data but never
/// writes (plugin-sql owns writes — Rule: no DB writes in Rust). The store degrades to
/// empty reads until plugin-sql lazily creates the file on the frontend's first query.
///
/// The returned [`ApiHandle`] carries the port that was actually bound (see
/// [`ApiHandle::port`]); callers should advertise it via [`write_endpoint_file`].
pub async fn spawn(db_path: PathBuf) -> AppResult<ApiHandle> {
    let host = Ipv4Addr::LOCALHOST;
    let desired = desired_port();
    let listener = bind_with_fallback(host, desired).await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!(
            "could not bind any local API port near {desired}: {e}"
        ))
    })?;
    let addr: SocketAddr = listener
        .local_addr()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("reading bound API addr: {e}")))?;
    let port = addr.port();

    let state = AppState::new(Arc::new(DeferredReadOnlyStore::new(db_path)));
    let app = router(state).layer(permissive());

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

    Ok(ApiHandle {
        shutdown: Some(tx),
        port,
    })
}

/// Best-effort: write `{ "port", "base_url" }` into `data_dir/api-endpoint.json` so external
/// integrations can find the API after auto-fallback moved it off the default port. Failure
/// is logged, never fatal — the API is already serving regardless.
pub fn write_endpoint_file(data_dir: &Path, port: u16) {
    let path = data_dir.join(ENDPOINT_FILENAME);
    // Hand-rolled JSON: the only interpolated value is a `u16`, so there is nothing to escape.
    let body =
        format!("{{\n  \"port\": {port},\n  \"base_url\": \"http://127.0.0.1:{port}\"\n}}\n");
    if let Err(e) = std::fs::write(&path, body) {
        warn!("could not write API endpoint file {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_desired_port_handles_all_cases() {
        assert_eq!(parse_desired_port(None), DEFAULT_API_PORT);
        assert_eq!(parse_desired_port(Some(" 9000 ".into())), 9000);
        assert_eq!(
            parse_desired_port(Some("not-a-port".into())),
            DEFAULT_API_PORT
        );
        assert_eq!(parse_desired_port(Some("0".into())), DEFAULT_API_PORT);
        assert_eq!(parse_desired_port(Some("70000".into())), DEFAULT_API_PORT);
    }

    #[tokio::test]
    async fn binds_desired_port_when_free() {
        let host = Ipv4Addr::LOCALHOST;
        let listener = bind_with_fallback(host, 23500).await.unwrap();
        assert_eq!(listener.local_addr().unwrap().port(), 23500);
    }

    #[tokio::test]
    async fn falls_back_when_desired_is_busy() {
        let host = Ipv4Addr::LOCALHOST;
        let base = 23000;
        // Occupy the desired port for the duration of the test.
        let _occupied = tokio::net::TcpListener::bind((host, base)).await.unwrap();
        let listener = bind_with_fallback(host, base).await.unwrap();
        assert_ne!(listener.local_addr().unwrap().port(), base);
    }

    #[test]
    fn endpoint_file_is_written_with_bound_port() {
        let dir =
            std::env::temp_dir().join(format!("shalgalt-endpoint-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        write_endpoint_file(&dir, 22345);
        let body = std::fs::read_to_string(dir.join(ENDPOINT_FILENAME)).unwrap();
        assert!(body.contains("\"port\": 22345"));
        assert!(body.contains("http://127.0.0.1:22345"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
