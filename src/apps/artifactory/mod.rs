mod router;

pub use router::{add_artifactory, list_all_artifactory, modify_artifactory, remove_artifactory};

pub use crate::utils::AppState;
use axum::{
    Router,
    routing::{delete, get, patch, post},
};
// use router::add_artifactory;
use tokio::sync::RwLock;

pub fn artifacotry_routes(state: std::sync::Arc<RwLock<Option<AppState>>>) -> Router {
    Router::new()
        .route("/", post(add_artifactory))
        .route("/", get(list_all_artifactory))
        .route("/", delete(remove_artifactory))
        .route("/", patch(modify_artifactory))
        .with_state(state)
}
