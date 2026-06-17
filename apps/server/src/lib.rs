//! `shalgalt-server` — standalone HTTP host for the `/v1/` REST surface (P5-05).
//!
//! Reuses `shalgalt_core::api::router` verbatim (Rule 4 — the router is never forked) over
//! a read-write `shalgalt_store::SqliteStore`, and layers Server-mode concerns on top:
//! bearer auth (from `SHALGALT_API_TOKEN`) and a CORS allow-list (never `Any`). The binary
//! owns the whole process, so shutdown is a simple SIGINT/SIGTERM handler.
//!
//! App assembly lives in [`build_app`] so the integration test can exercise the auth path
//! without binding a socket.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use axum::http::HeaderValue;
use axum::Router;
use clap::Parser;
use shalgalt_core::api::{auth, cors, router, AppState};
use shalgalt_store::SqliteStore;
use tracing::{info, warn};

/// Environment variable carrying the bearer token. When unset/empty the API is served
/// unauthenticated (with a loud warning) — convenient for a trusted LAN, dangerous on an
/// open network.
pub const TOKEN_ENV: &str = "SHALGALT_API_TOKEN";

#[derive(Debug, Parser)]
#[command(
    name = "shalgalt-server",
    version,
    about = "Standalone shalgalt-omr HTTP API"
)]
pub struct Cli {
    /// Address to bind, e.g. `0.0.0.0:8080`.
    #[arg(long, default_value = "0.0.0.0:8080")]
    pub bind: SocketAddr,

    /// Path to the SQLite database file (created and migrated if absent).
    #[arg(long)]
    pub db: PathBuf,

    /// Allowed CORS origin. Repeat for several; omit to disable cross-origin browser
    /// access entirely.
    #[arg(long = "allow-origin", value_name = "ORIGIN")]
    pub allow_origin: Vec<String>,
}

/// Assemble the router with CORS + optional bearer auth layered on.
///
/// Layer order matters: auth is applied first so CORS ends up *outermost* and answers the
/// preflight `OPTIONS` (which carries no `Authorization`) before the auth guard runs.
pub fn build_app(state: AppState, allow_origins: &[String], token: Option<String>) -> Router {
    let mut app = router(state);

    // Distinguish "absent" from "present-but-empty" — `SHALGALT_API_TOKEN=` (a common Docker
    // mistake) must not be mistaken for "unset" in the log.
    match token {
        Some(t) if !t.is_empty() => {
            app = auth::with_bearer(app, t);
            info!("bearer auth enabled");
        }
        Some(_) => warn!("{TOKEN_ENV} is set but empty — the API is UNAUTHENTICATED"),
        None => warn!("{TOKEN_ENV} not set — the API is UNAUTHENTICATED"),
    }

    if allow_origins.is_empty() {
        warn!("no --allow-origin given — cross-origin browser requests will be blocked");
    }
    // Surface origins the CORS layer will accept into its list but can never actually match,
    // so a misconfiguration is not invisible. A browser `Origin:` header is scheme + host +
    // optional port with no path/query, and tower-http compares it byte-exactly — so an
    // entry that fails to parse as a header value, or carries a path/query, is dead config.
    for origin in allow_origins.iter().filter(|o| !is_matchable_origin(o)) {
        warn!("--allow-origin {origin:?} can never match a browser Origin header (bad value or has a path); it will never grant access");
    }

    app.layer(cors::allow_list(allow_origins))
}

/// Whether `origin` could ever equal a real browser `Origin:` header: a valid header value,
/// `scheme://authority`, with nothing after the authority.
fn is_matchable_origin(origin: &str) -> bool {
    if origin.parse::<HeaderValue>().is_err() {
        return false;
    }
    match origin.split_once("://") {
        Some((scheme, rest)) => !scheme.is_empty() && !rest.is_empty() && !rest.contains('/'),
        None => false,
    }
}

/// Open + migrate the database, build the app, and serve until SIGINT/SIGTERM.
pub async fn serve(cli: Cli) -> anyhow::Result<()> {
    let store = SqliteStore::open_read_write(&cli.db)
        .with_context(|| format!("opening database {:?}", cli.db))?;
    store.migrate().context("migrating database")?;

    let state = AppState::new(Arc::new(store));
    let token = std::env::var(TOKEN_ENV).ok();
    let app = build_app(state, &cli.allow_origin, token);

    let listener = tokio::net::TcpListener::bind(cli.bind)
        .await
        .with_context(|| format!("binding {}", cli.bind))?;
    info!("shalgalt-server listening on http://{}", cli.bind);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;
    Ok(())
}

/// Initialize the tracing subscriber. Matches the desktop's `fmt` + `EnvFilter` shape;
/// defaults to `info` when `RUST_LOG` is unset.
pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .try_init();
}

/// Resolve when the process receives SIGINT (Ctrl-C) or SIGTERM (systemd/Docker stop).
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::is_matchable_origin;

    #[test]
    fn accepts_scheme_host_port_origins() {
        assert!(is_matchable_origin("http://localhost:5173"));
        assert!(is_matchable_origin("https://app.example.com"));
    }

    #[test]
    fn rejects_origins_with_a_path_or_no_scheme() {
        assert!(!is_matchable_origin("https://app.example.com/path"));
        assert!(!is_matchable_origin("https://app.example.com/"));
        assert!(!is_matchable_origin("app.example.com"));
        assert!(!is_matchable_origin(""));
    }
}
