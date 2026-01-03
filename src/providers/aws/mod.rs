use crate::builders;
use crate::models::DevBox;
use crate::providers::DevBoxProvider;
use crate::utils::artifactory::{self, Artifactory};
mod container;

#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct AwsProvider {
    access_key: String,
    secret_key: String,
    region: String,
    artifactory: Artifactory,
}

impl AwsProvider {
    pub fn new(config: serde_json::Value, artifactory: Artifactory) -> Self {
        AwsProvider {
            access_key: config.get("access_key").unwrap().to_string(),
            secret_key: config.get("secret_key").unwrap().to_string(),
            region: config.get("region").unwrap().to_string(),
            artifactory,
        }
    }
}

#[async_trait::async_trait]
impl DevBoxProvider for AwsProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create_devbox(
        &mut self,
        builder: &str,
        devcontainer: DevBox,
        path: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        todo!("Implement this feature later");
    }

    async fn delete_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }

    async fn start_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }

    async fn stop_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }

    async fn get_status(&self, id: String) -> Result<String, Box<dyn std::error::Error>> {
        todo!("To be implemented")
    }
}
