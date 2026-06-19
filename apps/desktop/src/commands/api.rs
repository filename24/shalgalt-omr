//! Read-only IPC exposing where the embedded API server bound (ADR 0015 port hardening).
//!
//! Because auto-fallback can move the API off its default port, the frontend (and any
//! "developer info" surface) needs a way to learn the live port. This command returns it,
//! plus a ready-to-use loopback base URL. When the API failed to start at all, `running` is
//! `false` and the port fields are `null` — callers must handle that gracefully rather than
//! assume a fixed port.

use serde::Serialize;

/// Managed state holding the embedded API's bound port, or `None` when it never started.
/// Registered once during bootstrap (both the success and failure paths) so the
/// [`api_info`] command always has something to read.
#[derive(Clone)]
pub struct ApiInfo {
    port: Option<u16>,
}

impl ApiInfo {
    /// The API is serving on `port`.
    pub fn running(port: u16) -> Self {
        Self { port: Some(port) }
    }

    /// The API did not start (e.g. no free port, or a non-recoverable bind error).
    pub fn disabled() -> Self {
        Self { port: None }
    }

    fn dto(&self) -> ApiInfoDto {
        ApiInfoDto {
            running: self.port.is_some(),
            port: self.port,
            base_url: self.port.map(|p| format!("http://127.0.0.1:{p}")),
        }
    }
}

/// Wire shape returned to the webview.
#[derive(Serialize, Clone)]
pub struct ApiInfoDto {
    /// Whether the embedded HTTP API is currently serving.
    pub running: bool,
    /// The loopback port it bound, or `null` when not running.
    pub port: Option<u16>,
    /// Convenience `http://127.0.0.1:<port>` base, or `null` when not running.
    pub base_url: Option<String>,
}

/// Report the embedded API's live address (or that it is disabled).
#[tauri::command]
pub fn api_info(info: tauri::State<'_, ApiInfo>) -> ApiInfoDto {
    info.dto()
}
