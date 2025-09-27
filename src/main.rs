use axum::{
    extract::Query,
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use crate::common::{list_containers, create_container, fetch_container, Container};
use serde_json::json;
use tracing::error;
use tracing_subscriber;
use http::{HeaderValue, Method, HeaderName};
mod common;

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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// GET /container/list
async fn all_containers() -> Result<Json<serde_json::Value>, StatusCode> {
    match list_containers() {
        Ok(json_string) => {
            let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap_or(json!([]));
            Ok(Json(parsed))
        }
        Err(e) => {error!("list_container failed: {:?}", e); Err(StatusCode::INTERNAL_SERVER_ERROR)},
    }
}

// POST /container/create
async fn new_container(Json(payload): Json<Container>) -> StatusCode {
    match create_container(payload) {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {error!("create_container failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR},
    }
}

// GET /container/get
async fn get_container(Query(params): Query<ContainerQuery>) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = params.name;
    match fetch_container(name) {
         Ok(json_string) => {
            let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap_or(json!([]));
            Ok(Json(parsed))
        }
        Err(e) => {error!("get_container failed: {:?}", e); Err(StatusCode::INTERNAL_SERVER_ERROR)},
    }
}

#[derive(serde::Deserialize)]
struct ContainerQuery {
    name: String,
}