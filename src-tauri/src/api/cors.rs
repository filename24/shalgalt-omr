//! CORS 정책. P0에서는 개발 편의를 위해 모든 origin/method/header를 허용한다.
//! P4에서 운영 정책으로 좁힌다.

use tower_http::cors::{Any, CorsLayer};

pub fn permissive() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
