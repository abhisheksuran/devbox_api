mod action;
mod container;

use crate::db::{delete_container, insert_container, update_container_status};
use crate::models::DevBox;
use crate::providers::DevBoxProvider;
use crate::task_log;
use crate::{logs::ASYNC_TASK_ID, utils::artifactory::Artifactory};

pub use action::handle_exec_stream;
use bollard::Docker;
use container::{create, exec, remove, start, status, stop};
// use image::create_image;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct DockerProvider {
    pub artifactory: Option<Artifactory>,
    connection: Arc<Docker>,
}

impl DockerProvider {
    pub fn new(connection: Arc<Docker>, artifactory: Option<Artifactory>) -> Self {
        DockerProvider {
            connection,
            artifactory,
        }
    }

    pub fn get_connection(&self) -> Arc<Docker> {
        self.connection.clone()
    }
}

#[async_trait::async_trait]
impl DevBoxProvider for DockerProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create_devbox(
        &mut self,
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

        let task_id = ASYNC_TASK_ID.with(|id| id.clone());
        let t_id = task_id.clone();
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

        let status = match devcontainer.start_on_create {
            Some(_s) => {
                if _s {
                    "running"
                } else {
                    "created"
                }
            }
            None => "created",
        };
        insert_container("docker", &devcontainer.name, status, &t_id, &id.clone()).await?;
        Ok(json!({ "status": "success", "container_id": id }))
    }

    async fn delete_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        let docker = self.connection.clone();
        match remove(docker, id.clone()).await {
            Ok(()) => (),
            _ => return Err("Fail to delete container".into()),
        };
        delete_container(id).await?;
        Ok(())
    }

    async fn start_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        let docker = self.connection.clone();
        match start(docker, id.clone()).await {
            Ok(()) => (),
            _ => return Err("Fail to start container".into()),
        }

        update_container_status(&id, "running").await?;
        Ok(())
    }

    async fn stop_devbox(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        let docker = self.connection.clone();
        match stop(docker, id.clone()).await {
            Ok(()) => (),
            _ => return Err("Fail to stop container".into()),
        }
        update_container_status(&id, "exited").await?;
        Ok(())
    }

    async fn get_status(&self, id: String) -> Result<String, Box<dyn std::error::Error>> {
        let docker = self.connection.clone();
        status(docker, id).await
    }
}
