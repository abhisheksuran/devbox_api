use crate::providers::{
    DevBoxProvider, ProviderEnum, aws::AwsProvider, azure::AzureProvider, docker::DockerProvider,
};
use crate::utils::artifactory::Artifactory;
use bollard::Docker;
use std::sync::Arc;

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
        provider: ProviderEnum,
    ) -> Box<dyn DevBoxProvider + Send + Sync> {
        match provider {
            ProviderEnum::Docker => Box::new(self.docker.clone()),
            ProviderEnum::Azure => Box::new(self.azure.as_ref().unwrap().clone()),
            ProviderEnum::Aws => Box::new(self.aws.as_ref().unwrap().clone()),
        }
    }

    pub fn get_docker_connection(&self) -> Arc<Docker> {
        self.docker.connection.clone()
    }
}
