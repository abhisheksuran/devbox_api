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

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct AzureMod {
    tanent: String,
    subscription: String,
    token: String,
    location: String,
    resource_group: String,
    address_type: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct AwsMod {
    access_key: String,
    secret_key: String,
    region: String,
}

pub async fn add_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<ProviderMod>,
) -> impl IntoResponse {
    let success =
        (insert_provider(&data.provider.to_string(), &data.config.to_string()).await).is_ok();
    if success {
        update_appstate(state).await;
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO ADD PROVIDER").into_response()
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
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO UPDATE PROVIDER",
        )
            .into_response()
    }
}

pub async fn update_azure(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<AzureMod>,
) -> impl IntoResponse {
    let success = update_provider_db("azure", serde_json::to_string(&data).unwrap())
        .await
        .is_ok();
    if success {
        update_appstate(state).await;
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO UPDATE PROVIDER",
        )
            .into_response()
    }
}

pub async fn update_docker(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<crate::utils::TunnelConfig>,
) -> impl IntoResponse {
    let success = update_provider_db("docker", serde_json::to_string(&data).unwrap())
        .await
        .is_ok();
    if success {
        update_appstate(state).await;
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO UPDATE PROVIDER",
        )
            .into_response()
    }
}

pub async fn update_aws(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<AwsMod>,
) -> impl IntoResponse {
    let success = update_provider_db("aws", serde_json::to_string(&data).unwrap())
        .await
        .is_ok();
    if success {
        update_appstate(state).await;
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO UPDATE PROVIDER",
        )
            .into_response()
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
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO REMOVE PROVIDER",
        )
            .into_response()
    }
}

pub async fn list_all_providers() -> impl IntoResponse {
    match list_providers().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}")).into_response(),
    }
}
