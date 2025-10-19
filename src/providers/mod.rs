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

#[async_trait::async_trait]
pub trait Connection {}

#[async_trait::async_trait]
pub trait DevBoxProvider {
    async fn create_devbox(
        &self,
        devcontainer: Option<DevBox>,
        path: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;

    fn as_any(&self) -> &dyn std::any::Any;
}
