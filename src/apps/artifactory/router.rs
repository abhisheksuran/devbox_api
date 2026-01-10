use crate::apps::artifactory::model::ArtifactoryMod;
use crate::db::{delete_artifactory, insert_artifactory, list_artifactory, update_artifactory};
use crate::providers::ProviderModQuery;
use crate::utils::AppState;
use crate::utils::update_appstate;
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[utoipa::path(
    post,
    path = "/artifactories",
    request_body(
        content = ArtifactoryMod,
        description = "Add Artifactory for a provider",
    ),
    description = "Add Artifactory for a provider",
    responses(
        (status = 200, description = "Success", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_artifactory(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ArtifactoryMod>,
) -> impl IntoResponse {
    // update_appstate(state.clone()).await;
    let success = (insert_artifactory(&data.provider.to_string(), data.config).await).is_ok();
    if success {
        update_appstate(state).await;
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO ADD ARIFACTORY",
        )
            .into_response()
    }
}

#[utoipa::path(
    patch,
    path = "/artifactories",
    request_body(
        content = ArtifactoryMod,
        description = "Modify Artifactory for a provider",
    ),
    description = "Modify Artifactory for a provider",
    responses(
        (status = 200, description = "Success", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn modify_artifactory(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ArtifactoryMod>,
) -> impl IntoResponse {
    let success = update_artifactory(&data.provider.to_string(), data.config)
        .await
        .is_ok();
    if success {
        update_appstate(state).await;
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO UPDATE ARIFACTORY",
        )
            .into_response()
    }
}

#[utoipa::path(
    delete,
    path = "/artifactories",
    params(("provider" = ProviderModQuery, Query, description = "Provider for which artifactory needs to be deleted")),
    description = "Delete current artifactory for the provider",
    responses(
        (status = 200, description = "Task accepted", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn remove_artifactory(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Query(provider): Query<ProviderModQuery>,
) -> impl IntoResponse {
    let success = delete_artifactory(&provider.provider.to_string())
        .await
        .is_ok();
    if success {
        update_appstate(state).await;
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO REMOVE ARIFACTORY",
        )
            .into_response()
    }
}

#[utoipa::path(
    get,
    path = "/artifactories",
    description = "List all artifactories",
    responses(
        (status = 200, description = "Success", body = Vec<(String, String, String, String, String, String)>),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_all_artifactory() -> impl IntoResponse {
    match list_artifactory().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}")).into_response(),
    }
}
