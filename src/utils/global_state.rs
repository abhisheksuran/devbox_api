use crate::db::get_latest_config;
use crate::providers::{
    DevBoxProvider, ProviderEnum, aws::AwsProvider, azure::AzureProvider, docker::DockerProvider,
};
use crate::utils::artifactory::Artifactory;
use bollard::Docker;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, warn};

#[derive(Clone)]
pub struct AppState {
    pub docker: DockerProvider,
    pub azure: Option<AzureProvider>,
    pub aws: Option<AwsProvider>,
    pub log_storage_path: String,
}

impl AppState {
    pub fn get_provider_strategy(
        &self,
        provider: &ProviderEnum,
    ) -> Box<dyn DevBoxProvider + Send + Sync> {
        match provider {
            ProviderEnum::Docker => Box::new(self.docker.clone()),
            ProviderEnum::Azure => Box::new(self.azure.as_ref().unwrap().clone()),
            ProviderEnum::Aws => Box::new(self.aws.as_ref().unwrap().clone()),
        }
    }

    pub fn get_docker_connection(&self) -> Arc<Docker> {
        self.docker.get_connection()
    }

    pub async fn get_state(
        state: Arc<RwLock<Option<AppState>>>,
    ) -> Result<AppState, Box<dyn std::error::Error>> {
        let guard = state.read().await;

        let state_clone = match &*guard {
            Some(state) => Ok(state.clone()),
            None => {
                return Err("AppState not initialized".into());
            }
        };
        state_clone
    }
}

pub async fn update_appstate(app_state: Arc<RwLock<Option<AppState>>>) {
    let mut write_guard = app_state.write().await;

    let previous_state = write_guard.take(); // Option<AppState>

    let provider_data = match get_latest_config().await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to obtain provider config: {e}");
            *write_guard = previous_state;
            return;
        }
    };

    let mut azure_opt: Option<AzureProvider> = None;
    let mut aws_opt: Option<AwsProvider> = None;
    let mut docker_art_opt: Option<Artifactory> = None;

    for (provider, cfg, art) in provider_data {
        match provider.as_str() {
            "azure" => azure_opt = Some(AzureProvider::new(cfg, art)),
            "aws" => aws_opt = Some(AwsProvider::new(cfg, art)),
            "docker" => docker_art_opt = Some(art),
            other => warn!("Unknown provider `{other}` – ignored"),
        }
    }

    let docker_conn = match &previous_state {
        Some(prev) => prev.docker.get_connection().into(),
        None => Arc::new(bollard::Docker::connect_with_local_defaults().unwrap()),
    };

    let new_state = AppState {
        docker: DockerProvider::new(docker_conn, docker_art_opt),
        azure: azure_opt,
        aws: aws_opt,
        log_storage_path: previous_state.unwrap().log_storage_path,
    };

    *write_guard = Some(new_state);
}
