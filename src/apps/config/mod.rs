mod handler;
mod router;
pub use handler::*;
pub use router::*;

use crate::providers::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::RwLock;

pub fn config_routes(state: std::sync::Arc<RwLock<Option<AppState>>>) -> Router {
    Router::new()
        .route("/edit", get(update_state))
        .with_state(state)
}
