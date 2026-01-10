use crate::apps::provider::model::{AwsMod, AzureMod, ProviderMod, RemoteConfig};
use crate::db::{delete_provider, insert_provider, list_providers, update_provider_db};
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

#[utoipa::path(
    patch,
    path = "/providers",
    request_body(
        content = ProviderMod,
        description = "Modify any provider",
    ),
    description = "Modify any provider with expected json for a provider",
    responses(
        (status = 200, description = "Success", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
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

#[utoipa::path(
    patch,
    path = "/providers/azure",
    request_body(
        content = AzureMod,
        description = "Modify Azure provider",
    ),
    description = "Modify Azure Provider",
    responses(
        (status = 200, description = "Success", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
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

#[utoipa::path(
    patch,
    path = "/providers/docker",
    request_body(
        content = RemoteConfig,
        description = "Modify Docker provider",
    ),
    description = "Modify Docker provider",
    responses(
        (status = 200, description = "Success", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_docker(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(data): Json<RemoteConfig>,
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

#[utoipa::path(
    patch,
    path = "/providers/aws",
    request_body(
        content = AwsMod,
        description = "Modify aws provider",
    ),
    description = "Modify aws provider",
    responses(
        (status = 200, description = "Success", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
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

#[utoipa::path(
    get,
    path = "/providers",
    description = "List all available providers",
    responses(
        (status = 200, description = "Task accepted", body = Vec<(String, String)>),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_all_providers() -> impl IntoResponse {
    match list_providers().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}")).into_response(),
    }
}
