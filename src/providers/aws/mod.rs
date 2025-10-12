use crate::models::DevBox;
use crate::providers::DevBoxProvider;
use bollard::Docker;
use std::sync::Arc;

pub struct AwsProvider;

#[async_trait::async_trait]
impl DevBoxProvider for AwsProvider {
    async fn create_devbox(
        &self,
        docker: Arc<Docker>,
        devcontainer: DevBox,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        todo!("Implement this feature later");
    }
}
