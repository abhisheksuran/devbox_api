use axum::{
    Router,
    routing::{get, post},
};
use bollard::Docker;
use http::{HeaderName, HeaderValue, Method};
use routes::{all_containers, get_container, new_container, websocket_exec_handler};
use std::sync::Arc;
mod db;
mod handlers;
mod models;
mod providers;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let docker = Arc::new(Docker::connect_with_local_defaults()?);
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin([HeaderValue::from_static("http://127.0.0.1:3000")])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([HeaderName::from_static("content-type")]);

    let app = Router::new()
        .route("/container/get", get(get_container))
        .route("/container/list", get(all_containers))
        .route("/container/create", post(new_container))
        .route("/ws/docker/{id}", get(websocket_exec_handler))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(docker.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
