mod aws;
mod azure;
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
pub trait DevBoxProvider {
    async fn create_devbox(
        &self,
        docker: Arc<Docker>,
        devcontainer: Option<DevBox>,
        path: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

pub fn get_provider_strategy(provider: ProviderEnum) -> Box<dyn DevBoxProvider + Send + Sync> {
    match provider {
        ProviderEnum::Docker => Box::new(DockerProvider),
        ProviderEnum::Azure => Box::new(AzureProvider),
        ProviderEnum::Aws => Box::new(AwsProvider),
    }
}
