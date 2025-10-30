mod handler;
mod model;
mod router;

pub use handler::*;
pub use model::*;
pub use router::{
    __path_new_devbox, action_on_devbox, list_all_devbox, new_devbox, websocket_exec_handler,
};

use crate::utils::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::RwLock;

pub fn devbox_routes(state: std::sync::Arc<RwLock<Option<AppState>>>) -> Router {
    Router::new()
        .route("/create", post(new_devbox))
        .route("/{id}", get(action_on_devbox))
        .route("/list", get(list_all_devbox))
        .route("/ws/docker/{id}", get(websocket_exec_handler))
        .with_state(state)
}
