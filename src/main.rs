use axum::{
    Router,
    routing::{get, post},
};
use http::{HeaderName, HeaderValue, Method};
use routes::{all_containers, get_container, new_container};
mod db;
mod handlers;
mod models;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin([HeaderValue::from_static("http://127.0.0.1:3000")])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([HeaderName::from_static("content-type")]);

    let app = Router::new()
        .route("/container/get", get(get_container))
        .route("/container/list", get(all_containers))
        .route("/container/create", post(new_container))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
