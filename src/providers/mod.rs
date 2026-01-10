#![allow(clippy::to_string_trait_impl)]

pub mod aws;
pub mod azure;
pub mod docker;
use crate::models::DevBox;

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ProviderEnum {
    Docker,
    Azure,
    Aws,
}
#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ProviderModQuery {
    pub provider: ProviderEnum,
}

impl From<String> for ProviderEnum {
    fn from(s: String) -> Self {
        match s.as_str() {
            "aws" => ProviderEnum::Aws,
            "azure" => ProviderEnum::Azure,
            _ => ProviderEnum::Docker,
        }
    }
}

impl ToString for ProviderEnum {
    fn to_string(&self) -> String {
        match self {
            ProviderEnum::Docker => "docker".to_string(),
            ProviderEnum::Azure => "azure".to_string(),
            ProviderEnum::Aws => "aws".to_string(),
        }
    }
}

#[async_trait::async_trait]
pub trait DevBoxProvider {
    async fn create_devbox(
        &mut self,
        builder: &str,
        devcontainer: DevBox,
        path: String,
    ) -> Result<String, Box<dyn std::error::Error>>;

    async fn delete_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    async fn start_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    async fn stop_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    async fn get_status(&self, id: String) -> Result<String, Box<dyn std::error::Error>>;

    fn as_any(&self) -> &dyn std::any::Any;
}
