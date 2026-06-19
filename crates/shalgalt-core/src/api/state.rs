//! Shared router state: a handle to the host's [`DataStore`].
//!
//! axum requires state to be `Clone + Send + Sync + 'static`; wrapping the trait object in
//! an `Arc` satisfies all four while letting every handler share one store.

use std::sync::Arc;

use super::store::DataStore;

/// State threaded through every `/v1/` handler via axum's typed-state extractor.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn DataStore>,
}

impl AppState {
    pub fn new(store: Arc<dyn DataStore>) -> Self {
        Self { store }
    }
}
