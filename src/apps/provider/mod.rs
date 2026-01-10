pub mod model;
mod router;

pub use router::{
    add_provider, list_all_providers, modify_providers, remove_provider, update_aws, update_azure,
    update_docker,
};

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
        .route("/docker", patch(update_docker))
        .route("/azure", patch(update_azure))
        .route("/aws", patch(update_aws))
        .with_state(state)
}
