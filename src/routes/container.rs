use crate::handlers::{create_container, create_devbox, fetch_container, list_containers};
use crate::models::{Container, DevBox};
use crate::providers::ProviderEnum;
use crate::providers::docker::handle_exec_stream;
use axum::{
    Json,
    extract::{Path, Query, State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use bollard::Docker;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

// GET /container/list
pub async fn all_containers() -> Result<Json<serde_json::Value>, StatusCode> {
    match list_containers().await {
        Ok(json_string) => {
            let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap_or(json!([]));
            Ok(Json(parsed))
        }
        Err(e) => {
            error!("list_container failed: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// POST /container/create
pub async fn new_container(Json(payload): Json<Container>) -> StatusCode {
    match create_container(payload).await {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            error!("create_container failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// GET /container/get
pub async fn get_container(
    Query(params): Query<ContainerQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = params.name;
    match fetch_container(name).await {
        Ok(json_string) => {
            let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap_or(json!([]));
            Ok(Json(parsed))
        }
        Err(e) => {
            error!("get_container failed: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// pub async fn new_devbox(
//     Query(params): Query<ProviderQuery>,
//     State(docker): State<Arc<Docker>>,
//     Json(devcontainer): Json<DevBox>,
// ) -> Result<Json<serde_json::Value>, StatusCode> {
//     let provider = params.provider;
//     match create_devbox(provider, docker, devcontainer).await {
//         Ok(json_string) => {
//             let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap_or(json!([]));
//             Ok(Json(parsed))
//         }
//         Err(e) => {
//             error!("Creating new devbox failed: {:?}", e);
//             Err(StatusCode::INTERNAL_SERVER_ERROR)
//         }
//     }
// }

pub async fn new_devbox(
    ws: WebSocketUpgrade,
    Query(params): Query<ProviderQuery>,
    State(docker): State<Arc<Docker>>,
    // Json(devcontainer): Json<DevBox>,
) -> impl IntoResponse {
    let provider = params.provider;
    info!("websocket url hit for new devbox");
    ws.on_upgrade(move |socket| create_devbox(socket, provider, docker))
}

#[derive(serde::Deserialize)]
pub struct ProviderQuery {
    provider: ProviderEnum,
}

#[derive(serde::Deserialize)]
pub struct ContainerQuery {
    name: String,
}
// WS /ws/docker/{id}
pub async fn websocket_exec_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(docker): State<Arc<Docker>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_exec_stream(socket, docker, id))
}
