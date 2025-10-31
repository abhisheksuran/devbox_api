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

// impl TryFrom<String> for ProviderEnum {
//     type Error = String;

//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         match value.as_str() {
//             "aws" => Ok(ProviderEnum::Aws),
//             "azure" => Ok(ProviderEnum::Azure),
//             "docker" => Ok(ProviderEnum::Docker),
//             _ => Err(format!("Unknown provider: {}", value)),
//         }
//     }
// }

#[async_trait::async_trait]
pub trait DevBoxProvider {
    async fn create_devbox(
        &mut self,
        devcontainer: Option<DevBox>,
        path: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;

    async fn delete_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>>;

    async fn start_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>>;

    async fn stop_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>>;

    fn as_any(&self) -> &dyn std::any::Any;
}
