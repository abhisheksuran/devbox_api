use crate::models::DevBox;
use crate::providers::DevBoxProvider;
mod container;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct AwsProvider {
    access_key: String,
    secret_key: String,
    region: String,
}

#[async_trait::async_trait]
impl DevBoxProvider for AwsProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create_devbox(
        &mut self,
        devcontainer: Option<DevBox>,
        path: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        todo!("Implement this feature later");
    }

    async fn delete_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }

    async fn start_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }

    async fn stop_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }
}
