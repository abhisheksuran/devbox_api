use crate::apps::devbox::ActionEnum;
use crate::db::get_provider_and_id;
use crate::db::{
    delete_container, insert_container, update_container_resource_id, update_container_status,
};
use crate::logs::ASYNC_TASK_ID;
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
    let mut strategy = app_state.get_provider_strategy(&provider);

    let devcontainer = match devcontainer {
        Some(dc) => dc,
        None => DevBox::new(path.clone()).await,
    };
    let task_id = ASYNC_TASK_ID.with(|id| id.clone());
    let t_id = task_id.clone();
    insert_container(
        &provider.to_string(),
        &devcontainer.name,
        "Creating",
        &t_id,
        "NA",
    )
    .await?;
    let resource_id = strategy.create_devbox(devcontainer, path).await?;

    update_container_resource_id(&task_id, &resource_id).await?;
    update_container_status(&resource_id, "Created").await?;
    Ok(serde_json::json!({"Status": "Created"}))
}

pub async fn action_devbox(
    id: String,
    app_state: &AppState,
    action: ActionEnum,
) -> Result<(), Box<dyn std::error::Error>> {
    let (provider, resource_id) = get_provider_and_id(&id).await?;
    let strategy = app_state.get_provider_strategy(&provider);

    match action {
        ActionEnum::Delete => {
            strategy.delete_devbox(&resource_id).await?;
            delete_container(&resource_id).await?
        }
        ActionEnum::Start => {
            strategy.start_devbox(&resource_id).await?;
            update_container_status(&resource_id, "running").await?
        }
        ActionEnum::Stop => {
            strategy.stop_devbox(&resource_id).await?;
            update_container_status(&resource_id, "exited").await?
        }
    }
    Ok(())
}
