use crate::apps::config::update_provider;
use crate::providers::aws::AwsProvider;
use crate::providers::azure::AzureProvider;
use crate::providers::docker::DockerProvider;
use crate::providers::docker::DockerProviderMod;
use crate::utils::AppState;
use crate::utils::artifactory::Artifactory;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use bollard::Docker;
use std::sync::Arc;
use tokio::sync::RwLock;

#[utoipa::path(
    post,
    path = "/config/provider/azure/edit",
    description = "Configure Azure Provider",
    request_body = AzureProvider,
    responses(
        (status = 200, description = "Task accepted", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_azure_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(azure_provider): Json<AzureProvider>,
) -> impl IntoResponse {
    update_provider(state, azure_provider).await
}

#[utoipa::path(
    post,
    path = "/config/provider/aws/edit",
    description = "Configure AWS Provider",
    request_body = AwsProvider,
    responses(
        (status = 200, description = "Task accepted", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_aws_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(aws_provider): Json<AwsProvider>,
) -> impl IntoResponse {
    update_provider(state, aws_provider).await
}

#[utoipa::path(
    post,
    path = "/config/provider/docker/edit",
    description = "Configure Docker Provider",
    request_body = Artifactory,
    responses(
        (status = 200, description = "Task accepted", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_docker_provider(
    State(state): State<Arc<RwLock<Option<AppState>>>>,
    Json(docker_model): Json<DockerProviderMod>,
) -> impl IntoResponse {
    let docker_connection = Arc::new(Docker::connect_with_local_defaults().unwrap());

    let docker_provider = DockerProvider::new(
        docker_connection,
        docker_model.artifactory,
        docker_model.remote,
    );
    update_provider(state, docker_provider).await
}
