use axum::response::Response;
use bollard::Docker;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::providers::DevBoxProvider;
use crate::providers::aws::AwsProvider;
use crate::providers::azure::AzureProvider;
use crate::providers::docker::DockerProvider;
use crate::utils::AppState;
use crate::utils::artifactory::Artifactory;
use axum::extract::State;

pub async fn update_state(State(state): State<Arc<RwLock<Option<AppState>>>>) -> Response {
    let mut guard = state.write().await;

    match &mut *guard {
        Some(existing_state) => {
            existing_state.docker = DockerProvider {
                // Edit existing state
                connection: Arc::new(Docker::connect_with_local_defaults().unwrap()),
            };
        }
        None => {
            // default configs
            let new_state = AppState {
                docker: DockerProvider {
                    connection: Arc::new(Docker::connect_with_local_defaults().unwrap()),
                },
                azure: None,
                aws: None,
                log_storage_path: String::from("/var/log/devbox/"),
            };
            *guard = Some(new_state.clone());
        }
    }
    Response::default()
}

pub async fn update_provider(
    state: Arc<RwLock<Option<AppState>>>,
    provider: impl DevBoxProvider,
) -> Response {
    let mut azure_provider: Option<AzureProvider> = None;
    let mut aws_provider: Option<AwsProvider> = None;

    if let Some(azure) = provider.as_any().downcast_ref::<AzureProvider>() {
        azure_provider = Some(azure.clone());
    } else if let Some(aws) = provider.as_any().downcast_ref::<AwsProvider>() {
        aws_provider = Some(aws.clone());
    } else {
        // Unsupported provider
        return Response::default();
    }

    let mut guard = state.write().await;

    match &mut *guard {
        Some(existing_state) => {
            existing_state.azure = azure_provider;
            existing_state.aws = aws_provider;
        }
        None => {
            // default configs
            let new_state = AppState {
                docker: DockerProvider {
                    connection: Arc::new(Docker::connect_with_local_defaults().unwrap()),
                },
                azure: azure_provider,
                aws: aws_provider,
                log_storage_path: String::from("/var/log/devbox/"),
            };
            *guard = Some(new_state.clone());
        }
    }
    Response::default()
}
