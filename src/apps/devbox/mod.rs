mod handler;
mod model;
mod router;

pub use handler::*;
pub use model::*;
pub use router::{
    __path_action_on_devbox, __path_get_task_logs, __path_list_all_devbox, __path_new_devbox,
    action_on_devbox, get_details, get_task_logs, list_all_devbox, new_devbox,
    websocket_exec_handler,
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
        .route("/logs/{id}", get(get_task_logs))
        .route("/{id}", post(action_on_devbox))
        .route("/{id}", get(get_details))
        .route("/list", get(list_all_devbox))
        .route("/ws/docker/{id}", get(websocket_exec_handler))
        .with_state(state)
}
