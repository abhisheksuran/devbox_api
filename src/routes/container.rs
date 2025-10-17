use crate::handlers::{create_container, create_devbox, fetch_container, list_containers};
use crate::logs::{ASYNC_TASK_ID, TASK_LOGGERS, set_blocking_task_id};
use crate::models::{Container, DevBox};
use crate::providers::ProviderEnum;
use crate::providers::docker::handle_exec_stream;
use crate::task_log;
use axum::{
    Json,
    extract::{Path, Query, State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bollard::Docker;
use serde_json::json;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};
use utoipa::IntoParams;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use validator_derive::Validate;

const LOG_DIR: &str = "./data/task_logs";

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
    let path_param = params.path;
    let docker = docker.clone();
    // convert Option<Json<DevBox>> -> Option<DevBox>
    let devcontainer = devcontainer.clone().map(|j| j.0);

    // create task id and prepare per-task log writer

    let task_id = Uuid::new_v4().to_string();

    let _ = tokio::fs::create_dir_all(LOG_DIR).await;
    let log_path = format!("{}/{}.log", LOG_DIR, task_id);

    let (log_tx, mut log_rx) = tokio::sync::mpsc::channel::<String>(512);
    TASK_LOGGERS.insert(task_id.clone(), log_tx.clone());

    let log_path_clone = log_path.clone();
    tokio::spawn(async move {
        if let Ok(mut f) = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path_clone)
            .await
        {
            while let Some(line) = log_rx.recv().await {
                let mut out = line.clone();
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                let _ = f.write_all(out.as_bytes()).await;
                let _ = f.flush().await;
            }
        }
    });

    let taskid = task_id.clone();
    // Spawn blocking task with task_id set
    tokio::task::spawn_blocking(move || {
        set_blocking_task_id(taskid.clone());
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async move {
            task_log!("Starting devbox creation for provider: {:?}", provider);
            if let Err(e) = create_devbox(provider, docker, devcontainer, path_param).await {
                task_log!("Failed to create devbox: {}", e);
            }
            // TASK_LOGGERS.remove(&taskid);
        });
    });

    // Optional: log from async context
    // ASYNC_TASK_ID.scope(task_id.clone(), async {
    //     task_log!("Devbox task {} launched", task_id);
    // });

    (Json(json!({ "task_id": task_id }))).into_response()
}

#[derive(serde::Serialize, serde::Deserialize, Validate, IntoParams, ToSchema)]
pub struct ProviderQuery {
    provider: ProviderEnum,
    #[validate(custom = "validate_path")]
    path: String,
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
