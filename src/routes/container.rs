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
use utoipa::IntoParams;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use validator_derive::Validate;

// GET /container/list
#[utoipa::path(
    get,
    path = "/container/list",
    responses(
        (status = 200, description = "List containers", body = [Container]),
        (status = 500, description = "Internal server error")
    )
)]
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
#[utoipa::path(
    post,
    path = "/container/create",
    request_body = Container,
    responses(
        (status = 201, description = "Created"),
        (status = 500, description = "Internal server error")
    )
)]
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
#[utoipa::path(
    get,
    path = "/container/get",
    params(ContainerQuery),
    responses(
        (status = 200, description = "Get container", body = [Container]),
        (status = 500, description = "Internal server error")
    )
)]
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
#[utoipa::path(
    post,
    path = "/devbox/create",
    params(ProviderQuery),
    request_body = DevBox,
    responses(
        (status = 200, description = "Task accepted", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn new_devbox(
    Query(params): Query<ProviderQuery>,
    State(docker): State<Arc<Docker>>,
    devcontainer: Option<Json<DevBox>>,
) -> Response {
    // Support calls without a JSON body. If body present, validate it.
    if let Some(Json(dc)) = &devcontainer
        && !dc.is_valid()
    {
        error!("DevBox creation body is not valid");
        return error_response(
            StatusCode::BAD_REQUEST,
            "Invalid Creation, need either Image or Dockerfile",
        );
    }

    if let Err(e) = params.validate() {
        error!("Validation failed: {:?}", e);
        return error_response(StatusCode::BAD_REQUEST, e.to_string().as_str());
    }
    let provider = params.provider;
    let path_param = params.path.unwrap_or(".".to_string());
    let docker = docker.clone();
    // convert Option<Json<DevBox>> -> Option<DevBox>
    let devcontainer = devcontainer.clone().map(|j| j.0);

    tokio::task::spawn(async move {
        if let Err(e) = create_devbox(provider, docker, devcontainer, path_param).await {
            eprintln!("Failed to create devbox: {}", e);
        }
    });

    let id = Uuid::new_v4();
    (Json(json!({ "task_id": id.to_string() }))).into_response()
}

#[derive(serde::Deserialize, Validate, IntoParams, ToSchema)]
pub struct ProviderQuery {
    provider: ProviderEnum,
    #[validate(custom = "validate_path")]
    path: Option<String>,
}

fn validate_path(path: &str) -> Result<(), ValidationError> {
    let p = std::path::Path::new(path);

    if p.exists() && p.is_dir() {
        Ok(())
    } else {
        Err(ValidationError::new("Not a valid directory"))
    }
}

#[derive(serde::Deserialize, IntoParams, ToSchema)]
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
