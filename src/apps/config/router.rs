use crate::apps::config::update_provider;
use crate::providers::aws::AwsProvider;
use crate::providers::azure::AzureProvider;
use crate::utils::AppState;
use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn update_azure_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(azure_provider): Json<AzureProvider>,
) -> impl IntoResponse {
    update_provider(state, azure_provider).await
}

pub async fn update_aws_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(aws_provider): Json<AwsProvider>,
) -> impl IntoResponse {
    update_provider(state, aws_provider).await
}
