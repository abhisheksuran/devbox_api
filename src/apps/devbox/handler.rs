use crate::models::DevBox;
use crate::providers::AppState;
use crate::providers::ProviderEnum;
use std::sync::Arc;
use tracing::{error, info};

pub async fn create_devbox(
    provider: ProviderEnum,
    app_state: &AppState,
    devcontainer: Option<DevBox>,
    path: String,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let strategy = app_state.get_provider_strategy(provider);
    let result = strategy.create_devbox(devcontainer, path).await?;
    Ok(result)
}

// pub async fn remove_devbox(
//     id: String,
//     docker: Arc<Docker>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // Here you would implement the logic to remove the devbox/container
//     // For example, using the Docker API to stop and remove the container
//     // and then removing its record from the database.

//     // Placeholder implementation:
//     info!("Removing devbox with ID: {}", id);
// }
