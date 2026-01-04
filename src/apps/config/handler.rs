use axum::response::Response;
use bollard::Docker;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::{insert_artifactory, insert_builder, insert_provider, update_provider_db};
use crate::providers::DevBoxProvider;
use crate::providers::aws::AwsProvider;
use crate::providers::azure::AzureProvider;
use crate::providers::docker::DockerProvider;
use crate::utils::AppState;
use crate::utils::artifactory::Artifactory;
use axum::extract::State;

use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_LOG_DIR: &str = "/tmp/devbox";

fn get_log_path() -> Result<String, Box<dyn std::error::Error>> {
    // Get the system temp directory
    let mut temp_path: PathBuf = env::temp_dir();
    // Create a subdirectory inside temp
    temp_path.push("devbox");
    fs::create_dir_all(&temp_path)?;
    let path_str = temp_path.to_str().ok_or(DEFAULT_LOG_DIR)?.to_string();
    Ok(path_str)
}

#[utoipa::path(
    get,
    path = "/config/edit",
    responses(
        (status = 200, description = "Task accepted", body = String),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_state(State(state): State<Arc<RwLock<Option<AppState>>>>) -> Response {
    let mut guard = state.write().await;

    match &mut *guard {
        Some(existing_state) => {
            existing_state.docker = DockerProvider::new(
                Arc::new(Docker::connect_with_local_defaults().unwrap()),
                None,
                None,
            )
        }
        None => {
            // default configs
            let new_state = AppState {
                docker: DockerProvider::new(
                    Arc::new(Docker::connect_with_local_defaults().unwrap()),
                    None,
                    None,
                ),
                azure: None,
                aws: None,
                log_storage_path: get_log_path().unwrap_or(DEFAULT_LOG_DIR.to_string()),
                ssh_session: None,
                tunnel: None,
            };
            *guard = Some(new_state.clone());

            let _ = insert_provider("docker", "").await;
            let _ = insert_provider("azure", "").await;
            let _ = insert_provider("aws", "").await;
            let _ = insert_builder("remote1", 1, "docker", "{\"remote_ip\": \"192.168.175.129\",\"service_port\": 2375,\"username\": \"kali\",\"password\": \"kali\",\"local_port\": 2375}").await;
        }
    }
    Response::default()
}

pub async fn update_provider(
    state: Arc<RwLock<Option<AppState>>>,
    provider: impl DevBoxProvider,
) -> Response {
    let mut docker_provider: Option<DockerProvider> = None;
    let mut azure_provider: Option<AzureProvider> = None;
    let mut aws_provider: Option<AwsProvider> = None;

    if let Some(azure) = provider.as_any().downcast_ref::<AzureProvider>() {
        azure_provider = Some(azure.clone());
        let mut az_val = serde_json::to_value(azure).unwrap();
        let art: Artifactory =
            serde_json::from_value(az_val.get("artifactory").unwrap().clone()).unwrap();
        let _ = insert_artifactory(
            "azure",
            &art.server,
            &art.repository_name,
            art.username.as_deref().unwrap(),
            art.password.as_deref().unwrap(),
            "",
        )
        .await;
        az_val = az_val
            .as_object_mut()
            .unwrap()
            .remove("artifactory")
            .unwrap();
        update_provider_db("azure", serde_json::to_string(&az_val).unwrap())
            .await
            .unwrap();
    } else if let Some(aws) = provider.as_any().downcast_ref::<AwsProvider>() {
        aws_provider = Some(aws.clone());
        let mut aws_val = serde_json::to_value(aws).unwrap();
        let art: Artifactory =
            serde_json::from_value(aws_val.get("artifactory").unwrap().clone()).unwrap();
        let _ = insert_artifactory(
            "aws",
            &art.server,
            &art.repository_name,
            art.username.as_deref().unwrap(),
            art.password.as_deref().unwrap(),
            "",
        )
        .await;
        let aws_val = aws_val
            .as_object_mut()
            .unwrap()
            .remove("artifactory")
            .unwrap();
        update_provider_db("aws", serde_json::to_string(&aws_val).unwrap())
            .await
            .unwrap();
    } else if let Some(docker) = provider.as_any().downcast_ref::<DockerProvider>() {
        docker_provider = Some(docker.clone());
        let _ = insert_artifactory(
            "docker",
            &docker.artifactory.clone().unwrap().server,
            &docker.artifactory.clone().unwrap().repository_name,
            docker
                .artifactory
                .clone()
                .unwrap()
                .username
                .as_deref()
                .unwrap(),
            docker
                .artifactory
                .clone()
                .unwrap()
                .password
                .as_deref()
                .unwrap(),
            "",
        )
        .await;
        update_provider_db("docker", serde_json::to_string(&docker.remote).unwrap())
            .await
            .unwrap();
    } else {
        // Unsupported provider
        return Response::default();
    }

    let mut guard = state.write().await;

    match &mut *guard {
        Some(existing_state) => {
            existing_state.docker = docker_provider.unwrap_or_else(|| {
                DockerProvider::new(
                    Arc::new(Docker::connect_with_local_defaults().unwrap()),
                    None,
                    None,
                )
            });
            existing_state.azure = azure_provider;
            existing_state.aws = aws_provider;
        }
        None => {
            // default configs
            let new_state = AppState {
                docker: DockerProvider::new(
                    Arc::new(Docker::connect_with_local_defaults().unwrap()),
                    None,
                    None,
                ),
                azure: azure_provider,
                aws: aws_provider,
                log_storage_path: get_log_path().unwrap_or(DEFAULT_LOG_DIR.to_string()),
                ssh_session: None,
                tunnel: None,
            };
            *guard = Some(new_state.clone());
            let _ = insert_provider("docker", "").await;
            let _ = insert_provider("azure", "").await;
            let _ = insert_provider("aws", "").await;
        }
    }
    Response::default()
}
