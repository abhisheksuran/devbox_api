mod handler;
mod model;
mod router;

pub use handler::*;
pub use model::*;

use crate::providers::AppState;
use axum::{
    Router,
    routing::{get, post},
};
pub use router::{
    __path_all_containers, __path_get_container, __path_new_container, all_containers,
    get_container, new_container,
};
use tokio::sync::RwLock;

pub fn data_routes(state: std::sync::Arc<RwLock<Option<AppState>>>) -> Router {
    Router::new()
        .route("/get", get(get_container))
        .route("/list", get(all_containers))
        .route("/create", post(new_container))
        .with_state(state)
}
