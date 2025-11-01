use crate::apps::devbox::{
    ActionQuery, ProviderQuery, action_devbox, create_devbox, validate_state,
};
use crate::db::{insert_task, list_containers};
use crate::logs::{ASYNC_TASK_ID, TASK_LOGGERS};
use crate::models::DevBox;
use crate::providers::docker::handle_exec_stream;
use crate::task_log;
use crate::utils::AppState;
use axum::http::StatusCode;

use axum::{
    Json,
    extract::{Path, Query, State, ws::WebSocketUpgrade},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::error;

use uuid::Uuid;

const LOG_DIR: &str = "./data/task_logs";

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
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    devcontainer: Option<Json<DevBox>>,
) -> Response {
    // Validate request
    let validation_response =
        crate::apps::devbox::model::validator(&params, devcontainer.as_deref());
    if !validation_response.status().is_success() {
        return validation_response;
    }

    let provider = params.provider;
    let path_param = params.path;
    let path = path_param.clone();
    let state = state.clone();

    let is_state_initialized = validate_state(state.clone(), provider.clone()).await;
    if !is_state_initialized.status().is_success() {
        return is_state_initialized;
    }

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
    // tokio::task::spawn_blocking(move || {
    //     set_blocking_task_id(taskid.clone());
    //     let handle = tokio::runtime::Handle::current();
    //     handle.block_on(async move {
    //         task_log!("Starting devbox creation for provider: {:?}", provider);
    //         if let Err(e) = create_devbox(provider, state, devcontainer, path_param).await {
    //             task_log!("Failed to create devbox: {}", e);
    //         }
    //         // TASK_LOGGERS.remove(&taskid);
    //     });
    // });

    tokio::spawn({
        let state = state.clone(); // clone Arc, not AppState
        let provider = provider.clone();
        let taskid = taskid.clone();
        ASYNC_TASK_ID.scope(taskid.clone(), async move {
            task_log!("Starting devbox creation for provider: {:?}", provider);

            let guard = state.read().await;
            let app_state = match &*guard {
                Some(state) => state, // borrow AppState under lock
                None => {
                    error!("AppState is not initialized");
                    return;
                }
            };

            if let Err(e) = create_devbox(provider, app_state, devcontainer, path_param).await {
                task_log!("Failed to create devbox: {}", e);
            }

            // TASK_LOGGERS.remove(&taskid);
        })
    });

    // Optional: log from async context
    // ASYNC_TASK_ID.scope(task_id.clone(), async {
    //     task_log!("Devbox task {} launched", task_id);
    // });

    let _ = insert_task(&task_id, &provider.to_string(), "initiated", path).await;
    (Json(json!({ "task_id": task_id }))).into_response()
}

// WS /ws/docker/{id}
pub async fn websocket_exec_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(app_state): State<Arc<RwLock<Option<AppState>>>>,
) -> impl IntoResponse {
    let guard = app_state.read().await;

    let state = match &*guard {
        Some(state) => state.clone(),
        None => {
            error!("AppState is not initialized");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "AppState is not initialized",
            )
                .into_response();
        }
    };
    let docker = state.get_docker_connection();
    ws.on_upgrade(move |socket| handle_exec_stream(socket, docker, id))
}

pub async fn action_on_devbox(
    Path(id): Path<String>,
    Query(action): Query<ActionQuery>,
    State(state): State<Arc<RwLock<Option<AppState>>>>,
) -> impl IntoResponse {
    let guard = state.read().await;

    let state = match &*guard {
        Some(state) => state.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "AppState not initialized",
            ));
        }
    };
    let action = action.action;

    match action_devbox(id, &state, action).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to perform action",
        )),
    }
}

pub async fn list_all_devbox() -> impl IntoResponse {
    match list_containers().await {
        Ok(result) => {
            // let res = serde_json::to_string(&result);
            Ok(Json(result))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to list devbox")),
    }
}
