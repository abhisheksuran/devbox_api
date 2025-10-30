use crate::apps::devbox::ActionEnum;
use crate::db::DefaultDB;
use crate::models::DevBox;
use crate::providers::ProviderEnum;
use crate::utils::AppState;
use rusqlite::params;
use tracing::{error, info};

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

async fn get_provider_and_id(
    id: String,
) -> Result<(ProviderEnum, String), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db().unwrap();
    let mut stmt = conn.prepare("SELECT provider, resource_id FROM containers WHERE id = ?")?;
    let result = stmt.query_row(params![id], |row| {
        let provider: String = row.get(0)?;
        let resource_id: String = row.get(1)?;
        Ok((ProviderEnum::from(provider), resource_id))
    })?;
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

pub async fn list_devbox() -> Result<String, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;

    let mut stmt = conn.prepare("SELECT id, name, provider, status FROM containers")?;
    let container_iter = stmt.query_map([], |row| {
        let container_id: i32 = row.get(0)?;
        let name: String = row.get(1)?;
        let provider: String = row.get(2)?;
        let status: String = row.get(3)?;
        Ok((container_id, name, provider, status))
    })?;

    let containers: Result<Vec<(i32, String, String, String)>, _> = container_iter.collect();
    let json = serde_json::to_string(&containers?)?;
    Ok(json)
}
