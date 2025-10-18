mod action;
mod container;
mod image;

use crate::logs::{ASYNC_TASK_ID, get_blocking_task_id};
use crate::models::DevBox;
use crate::providers::{AppState, DevBoxProvider};
use crate::task_log;

pub use action::handle_exec_stream;
use bollard::Docker;
use container::{create, exec, remove, start};
// use image::create_image;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct DockerProvider {
    pub connection: Arc<Docker>,
}

#[async_trait::async_trait]
impl DevBoxProvider for DockerProvider {
    async fn create_devbox(
        &self,
        devcontainer: Option<DevBox>,
        path: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let docker = self.connection.clone();

        // create_image(docker.clone(), &devcontainer).await?;
        let devcontainer = match devcontainer {
            Some(dc) => dc,
            None => DevBox::new(path.clone()).await,
        };
        devcontainer.create_image(path, docker.clone()).await?;
        let id = create(docker.clone(), &devcontainer).await?;
        if devcontainer.start_on_create.unwrap_or(false) {
            start(docker.clone(), id.clone()).await?;
        }

        // spawn a task that tails container logs and forwards to sender
        let docker_clone = docker.clone();
        let id_clone = id.clone();

        // let task_id = if get_blocking_task_id().is_some() {
        //     get_blocking_task_id().unwrap()
        // } else {
        //     String::new()
        // };

        // task_log!("Current blocking task ID: {}", task_id);

        let task_id = ASYNC_TASK_ID.with(|id| id.clone());
        tokio::spawn(ASYNC_TASK_ID.scope(task_id.clone(), async move {
            if let Err(e) = crate::providers::docker::container::attach_container_logs(
                docker_clone,
                id_clone,
                task_id.clone(),
            )
            .await
            {
                task_log!("attach_container_logs failed: {:?}", e);
            }
        }));

        if let Some(script) = devcontainer.post_start_script.clone() {
            exec(docker, id.clone(), Some(script)).await?;
        }

        Ok(json!({ "status": "success", "container_id": id }))
    }
}
