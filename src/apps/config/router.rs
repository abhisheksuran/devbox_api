use crate::apps::config::update_provider;
use crate::providers::aws::AwsProvider;
use crate::providers::azure::AzureProvider;
use crate::providers::docker::DockerProvider;
use crate::utils::AppState;
use crate::utils::artifactory::Artifactory;
use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use bollard::Docker;
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

pub async fn update_docker_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(docker_artifactory): Json<Artifactory>,
) -> impl IntoResponse {
    let docker_connection = Arc::new(Docker::connect_with_local_defaults().unwrap());

    let docker_provider = DockerProvider::new(docker_connection, Some(docker_artifactory));
    update_provider(state, docker_provider).await
}
