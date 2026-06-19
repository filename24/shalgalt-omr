//! Entry point for the standalone `shalgalt-server` binary (P5-05).
//!
//! All assembly logic lives in the sibling library crate so it stays unit-testable; this
//! file only parses the CLI and hands off to [`shalgalt_server::serve`].

use clap::Parser;
use shalgalt_server::{serve, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shalgalt_server::init_tracing();
    serve(Cli::parse()).await
}
