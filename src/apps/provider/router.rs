use crate::db::{delete_provider, insert_provider, list_providers, update_provider_db};
use crate::providers::{ProviderEnum, ProviderModQuery};
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
pub struct ProviderMod {
    provider: ProviderEnum,
    config: serde_json::Value,
}

pub async fn add_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ProviderMod>,
) -> impl IntoResponse {
    let success =
        (insert_provider(&data.provider.to_string(), &data.config.to_string()).await).is_ok();
    if success {
        update_appstate(state).await;
        Ok((StatusCode::OK, "SUCCESS"))
    } else {
        Err((StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO ADD PROVIDER"))
    }
}

pub async fn modify_providers(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ProviderMod>,
) -> impl IntoResponse {
    let success = update_provider_db(&data.provider.to_string(), data.config.to_string())
        .await
        .is_ok();
    if success {
        update_appstate(state).await;
        Ok((StatusCode::OK, "SUCCESS"))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO UPDATE PROVIDER",
        ))
    }
}

pub async fn remove_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Query(provider): Query<ProviderModQuery>,
) -> impl IntoResponse {
    let success = delete_provider(&provider.provider.to_string())
        .await
        .is_ok();
    if success {
        update_appstate(state).await;
        Ok((StatusCode::OK, "SUCCESS"))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO REMOVE PROVIDER",
        ))
    }
}

pub async fn list_all_providers() -> impl IntoResponse {
    match list_providers().await {
        Ok(data) => Ok((StatusCode::OK, Json(data))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}"))),
    }
}
