mod router;

pub use router::{add_provider, list_all_providers, modify_providers, remove_provider};

pub use crate::utils::AppState;
use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use tokio::sync::RwLock;

pub fn provider_routes(state: std::sync::Arc<RwLock<Option<AppState>>>) -> Router {
    Router::new()
        // .route("/", post(add_provider))
        .route("/", get(list_all_providers))
        // .route("/", delete(remove_provider))
        .route("/", patch(modify_providers))
        .with_state(state)
}
