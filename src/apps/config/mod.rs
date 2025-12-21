mod handler;
mod router;
pub use handler::*;
pub use router::*;

use crate::utils::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::RwLock;

pub fn config_routes(state: std::sync::Arc<RwLock<Option<AppState>>>) -> Router {
    Router::new()
        .route("/edit", get(update_state))
        .route("/provider/azure/edit", post(update_azure_provider))
        .route("/provider/aws/edit", post(update_aws_provider))
        .route("/provider/docker/edit", post(update_docker_provider))
        .with_state(state)
}
