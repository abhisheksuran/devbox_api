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

pub async fn add_artifactory(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ArtifactoryMod>,
) -> impl IntoResponse {
    // update_appstate(state.clone()).await;
    let success = (insert_artifactory(&data.provider, data.config).await).is_ok();
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

pub async fn modify_artifactory(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ArtifactoryMod>,
) -> impl IntoResponse {
    let success = update_artifactory(&data.provider, data.config)
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

pub async fn list_all_artifactory() -> impl IntoResponse {
    match list_artifactory().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}")).into_response(),
    }
}
