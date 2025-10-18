use axum::response::Response;
use bollard::Docker;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::providers::AppState;
use crate::providers::aws::AwsProvider;
use crate::providers::azure::AzureProvider;
use crate::providers::docker::DockerProvider;
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
                azure: AzureProvider,
                aws: AwsProvider,
                artifactory: String::from("/var/lib/devbox/artifactory"),
                log_storage_path: String::from("/var/log/devbox/"),
            };
            *guard = Some(new_state.clone());
        }
    }
    Response::default()
}
