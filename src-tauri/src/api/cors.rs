//! CORS policy. P0 keeps the policy permissive (any origin / method / header) for ease of
//! development. P4 narrows it down before any production exposure.

use tower_http::cors::{Any, CorsLayer};

pub fn permissive() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
