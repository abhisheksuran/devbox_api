use crate::apps::data::ContainerQuery;
use crate::apps::data::{create_container, fetch_container, list_containers};
use crate::models::Container;
use axum::{Json, extract::Query, http::StatusCode};
use serde_json::json;

use tracing::{error, info};

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
