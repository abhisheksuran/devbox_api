pub mod aws;
pub mod azure;
pub mod docker;

use crate::models::DevBox;
use aws::AwsProvider;
use azure::AzureProvider;
use bollard::Docker;
use docker::DockerProvider;
use std::sync::Arc;

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ProviderEnum {
    Docker,
    Azure,
    Aws,
}

#[async_trait::async_trait]
pub trait Connection {}

#[async_trait::async_trait]
pub trait DevBoxProvider {
    async fn create_devbox(
        &self,
        devcontainer: Option<DevBox>,
        path: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

#[derive(Clone)]
pub struct AppState {
    pub docker: DockerProvider,
    pub azure: AzureProvider,
    pub aws: AwsProvider,
    pub artifactory: String,
    pub log_storage_path: String,
}

impl AppState {
    pub fn get_provider_strategy(
        &self,
        provider: ProviderEnum,
    ) -> Box<dyn DevBoxProvider + Send + Sync> {
        match provider {
            ProviderEnum::Docker => Box::new(self.docker.clone()),
            ProviderEnum::Azure => Box::new(self.azure.clone()),
            ProviderEnum::Aws => Box::new(self.aws.clone()),
        }
    }

    pub fn get_docker_connection(&self) -> Arc<Docker> {
        self.docker.connection.clone()
    }
}
