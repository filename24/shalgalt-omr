//! Dump the `/v1/` OpenAPI 3.1 document.
//!
//! The docs build (P7-03) feeds this JSON to `fumadocs-openapi`'s `generateFiles`, which
//! turns it into static MDX API-reference pages. Because the document comes from the same
//! `#[utoipa::path]` annotations the router serves at `/openapi.json`, the reference cannot
//! drift from the wiring.
//!
//! Usage:
//!   cargo run -p shalgalt-core --example dump_openapi               # write to stdout
//!   cargo run -p shalgalt-core --example dump_openapi -- out.json   # write to a file

use std::io::Write;

use shalgalt_core::api::ApiDoc;
use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(&ApiDoc::openapi())?;

    match std::env::args().nth(1) {
        Some(path) => {
            std::fs::write(&path, json.as_bytes())?;
            eprintln!("wrote OpenAPI document to {path}");
        }
        None => {
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(json.as_bytes())?;
            stdout.write_all(b"\n")?;
        }
    }

    Ok(())
}
