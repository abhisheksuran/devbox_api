pub mod model;
mod router;

pub use router::{
    __path_add_builder, __path_list_all_builders, __path_remove_builder, add_builder,
    list_all_builders, remove_builder,
};

pub use crate::utils::AppState;
use axum::{
    Router,
    routing::{delete, get, post},
};
use tokio::sync::RwLock;

pub fn builder_routes(state: std::sync::Arc<RwLock<Option<AppState>>>) -> Router {
    Router::new()
        .route("/", post(add_builder))
        .route("/", get(list_all_builders))
        .route("/", delete(remove_builder))
        .with_state(state)
}
