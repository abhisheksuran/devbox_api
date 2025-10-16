mod action;
mod container;
mod image;

use crate::models::DevBox;
use crate::providers::DevBoxProvider;

pub use action::handle_exec_stream;
use axum::extract::path;
use bollard::Docker;
use container::{create, exec, remove, start};
// use image::create_image;
use serde_json::json;
use std::sync::Arc;

pub struct DockerProvider;

#[async_trait::async_trait]
impl DevBoxProvider for DockerProvider {
    async fn create_devbox(
        &self,
        docker: Arc<Docker>,
        devcontainer: Option<DevBox>,
        path: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // create_image(docker.clone(), &devcontainer).await?;
        let devcontainer = match devcontainer {
            Some(dc) => dc,
            None => DevBox::new(path.clone()).await,
        };
        devcontainer
            .create_image(
                path, // TODO: change to a temp dir
                docker.clone(),
            )
            .await?;
        let id = create(docker.clone(), &devcontainer).await?;
        if devcontainer.start_on_create.unwrap_or(false) {
            start(docker.clone(), id.clone()).await?;
        }
        if let Some(script) = devcontainer.post_start_script.clone() {
            exec(docker, id.clone(), Some(script)).await?;
        }
        Ok(json!({ "status": "success", "container_id": id }))
    }
}
