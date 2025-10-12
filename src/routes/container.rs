use crate::handlers::{create_container, create_devbox, fetch_container, list_containers};
use crate::models::{Container, DevBox};
use crate::providers::ProviderEnum;
use crate::providers::docker::handle_exec_stream;
use axum::{
    Json,
    extract::{Path, Query, State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bollard::Docker;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

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

// POST /devbox/create?provider=docker|azure|aws
pub async fn new_devbox(
    Query(params): Query<ProviderQuery>,
    State(docker): State<Arc<Docker>>,
    Json(devcontainer): Json<DevBox>,
) -> Response {
    if !devcontainer.is_valid() {
        error!("DevBox creation body is not valid");
        return error_response(
            StatusCode::BAD_REQUEST,
            "Invalid Creation, need either Image or Dockerfile",
        );
    }

    let provider = params.provider;
    let docker = docker.clone();
    let devcontainer = devcontainer.clone();

    tokio::task::spawn(async move {
        if let Err(e) = create_devbox(provider, docker, devcontainer).await {
            eprintln!("Failed to create devbox: {}", e);
        }
    });

    let id = Uuid::new_v4();
    (Json(json!({ "task_id": id.to_string() }))).into_response()
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

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

// Delete /devbox/{id}
// pub async fn delete_devbox(
//     Path(id): Path<String>,
//     State(docker): State<Arc<Docker>>,
// ) -> StatusCode {
//     match crate::providers::docker::remove(docker.clone(), id).await {
//         Ok(_) => StatusCode::OK,
//         Err(e) => {
//             error!("delete_devbox failed: {:?}", e);
//             StatusCode::INTERNAL_SERVER_ERROR
//         }
//     }
// }
