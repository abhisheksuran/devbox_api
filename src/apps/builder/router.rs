use crate::apps::builder::model::{BuilderMod, BuilderModQuery};
use crate::db::{delete_builder, insert_builder, list_builders};
use crate::utils::TunnelConfig;
use axum::http::StatusCode;
use axum::{Json, extract::Query, response::IntoResponse};

#[utoipa::path(
    post,
    path = "/builders",
    request_body(
        content = BuilderMod,
        description = "Builder configuration 
                    If it's a remote builder config should be as below.
                    {remote_ip: String,
                    ssh_port: Optional integer,
                    service_port: integer,
                    private_key: Optional String,
                    passphrase: Optional String,
                    username: String,
                    password: Optional String,
                    local_port: integer}",
    ),
    description = "Add Builder to generate image. A builder can be docker, podman install on local machine or on remote machine",
    responses(
        (status = 200, description = "Success", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_builder(Json(data): Json<BuilderMod>) -> impl IntoResponse {
    let cfg = data.config.clone();

    if data.remote == 1 {
        match serde_json::from_value::<TunnelConfig>(cfg) {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Remote builder should have
                    remote_ip: String,
                    ssh_port: Optional integer,
                    service_port: integer,
                    private_key: Optional String,
                    passphrase: Optional String,
                    username: String,
                    password: Optional String,
                    local_port: integer,",
                )
                    .into_response();
            }
        };
    }

    let success = (insert_builder(
        &data.name,
        data.remote,
        &data.builder,
        &data.config.to_string(),
    )
    .await)
        .is_ok();

    if success {
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO ADD BUILDER").into_response()
    }
}

#[utoipa::path(
    delete,
    path = "/builders",
    params(("builder" = BuilderModQuery, Query, description = "Builder that needs to be deleted")),
    description = "Delete a builder",
    responses(
        (status = 200, description = "Task accepted", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn remove_builder(Query(builder): Query<BuilderModQuery>) -> impl IntoResponse {
    let success = delete_builder(&builder.builder.to_string()).await.is_ok();
    if success {
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO REMOVE BUILDER",
        )
            .into_response()
    }
}

#[utoipa::path(
    get,
    path = "/builders",
    description = "List all builders",
    responses(
        (status = 200, description = "Task accepted", body = Vec<(String, bool, String, String)>),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_all_builders() -> impl IntoResponse {
    match list_builders().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}")).into_response(),
    }
}
