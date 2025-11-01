use crate::db::{list_containers, update_container_status};
use crate::providers::DevBoxProvider;
use crate::utils::global_state::AppState;

pub async fn container_status_update(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let containers = list_containers().await?;
    for (_, _, provider, _, resource_id) in containers {
        let container_status = match provider.to_lowercase().as_str() {
            "docker" => state
                .docker
                .get_status(resource_id.clone())
                .await
                .unwrap_or("unknown".to_string()),
            "azure" => {
                if let Some(prd) = &state.azure {
                    prd.get_status(resource_id.clone())
                        .await
                        .unwrap_or("unknown".to_string())
                } else {
                    "unknown".to_string()
                }
            }
            "aws" => {
                if let Some(prd) = &state.aws {
                    prd.get_status(resource_id.clone())
                        .await
                        .unwrap_or("unknown".to_string())
                } else {
                    "unknown".to_string()
                }
            }
            _ => "unknown".to_string(),
        };
        update_container_status(&resource_id, &container_status).await?
    }
    Ok(())
}
