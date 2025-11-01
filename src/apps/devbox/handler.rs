use crate::apps::devbox::ActionEnum;
use crate::db::get_provider_and_id;
use crate::models::DevBox;
use crate::providers::ProviderEnum;
use crate::utils::AppState;
// use tracing::{error, info};

pub async fn create_devbox(
    provider: ProviderEnum,
    app_state: &AppState,
    devcontainer: Option<DevBox>,
    path: String,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut strategy = app_state.get_provider_strategy(provider);
    let result = strategy.create_devbox(devcontainer, path).await?;
    Ok(result)
}

pub async fn action_devbox(
    id: String,
    app_state: &AppState,
    action: ActionEnum,
) -> Result<(), Box<dyn std::error::Error>> {
    let (provider, resource_id) = get_provider_and_id(id).await?;
    let strategy = app_state.get_provider_strategy(provider);

    match action {
        ActionEnum::Delete => strategy.delete_devbox(resource_id).await,
        ActionEnum::Start => strategy.start_devbox(resource_id).await,
        ActionEnum::Stop => strategy.stop_devbox(resource_id).await,
    }
}
