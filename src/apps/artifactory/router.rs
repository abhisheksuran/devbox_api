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

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ArtifactoryMod {
    provider: String,
    server: String,
    repository_name: String,
    username: String,
    password: String,
    config: String,
}

// pub async fn add_artifactory(
//     State(state): State<Arc<RwLock<Option<AppState>>>>,
//     Json(data): Json<ArtifactoryMod>,
// ) -> impl IntoResponse {
//     match insert_artifactory(
//         &data.provider,
//         &data.server,
//         &data.repository_name,
//         &data.username,
//         &data.password,
//         &data.config,
//     )
//     .await
//     {
//         Ok(_) => {
//             let _ = update_appstate(state).await;
//             return Ok(Json(serde_json::json!({"MSG": "SUCCESS"})));
//         }
//         Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("ERROR: {e}"))),
//     }
// }

pub async fn add_artifactory(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ArtifactoryMod>,
) -> impl IntoResponse {
    // update_appstate(state.clone()).await;
    let success = (insert_artifactory(
        &data.provider,
        &data.server,
        &data.repository_name,
        &data.username,
        &data.password,
        &data.config,
    )
    .await)
        .is_ok();
    if success {
        update_appstate(state).await;
    };
}

pub async fn modify_artifactory(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ArtifactoryMod>,
) -> impl IntoResponse {
    let success = update_artifactory(
        &data.provider,
        &data.server,
        &data.repository_name,
        &data.username,
        &data.password,
        &data.config,
    )
    .await
    .is_ok();
    if success {
        update_appstate(state).await;
    };
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
    };
}

pub async fn list_all_artifactory() -> impl IntoResponse {
    match list_artifactory().await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}"))),
    }
}
