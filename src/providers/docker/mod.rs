mod action;
mod container;

use bollard::auth::DockerCredentials;
use futures_util::stream::TryStreamExt;

use crate::models::DevBox;
use crate::providers::DevBoxProvider;
use crate::task_log;
use crate::utils::{Remote, TunnelConfig};
use crate::{logs::ASYNC_TASK_ID, utils::artifactory::Artifactory};
pub use action::handle_exec_stream;
use bollard::Docker;
use bollard::query_parameters::CreateImageOptionsBuilder;
use container::{create, exec, remove, start, status, stop};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct DockerProviderMod {
    pub artifactory: Option<Artifactory>,
    pub remote: Option<TunnelConfig>,
}

#[derive(Clone)]
pub struct DockerProvider {
    pub artifactory: Option<Artifactory>,
    connection: Arc<Docker>,
    pub remote: Option<TunnelConfig>,
}

impl DockerProvider {
    pub fn new(
        connection: Arc<Docker>,
        artifactory: Option<Artifactory>,
        remote: Option<TunnelConfig>,
    ) -> Self {
        DockerProvider {
            connection,
            artifactory,
            remote,
        }
    }

    pub fn get_connection(&self) -> Arc<Docker> {
        self.connection.clone()
    }

    pub async fn refresh_connection(
        state: Arc<tokio::sync::RwLock<Option<crate::utils::AppState>>>,
        remote: Option<TunnelConfig>,
    ) -> Arc<Docker> {
        match remote.clone() {
            Some(cfg) => {
                let local_port = cfg.local_port;
                crate::utils::update_tunnel(state, cfg.service_port, cfg.local_port, Some(cfg))
                    .await;
                Arc::new(
                    bollard::Docker::connect_with_http(
                        format!("127.0.0.1:{}", local_port).as_str(),
                        10,
                        bollard::API_DEFAULT_VERSION,
                    )
                    .unwrap(),
                )
            }
            None => {
                crate::utils::update_tunnel(state, 0, 0, None).await;
                Arc::new(bollard::Docker::connect_with_local_defaults().unwrap())
            }
        }
    }

    pub async fn pull_image(
        &self,
        image_url: &str,
        tag: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let (_, repo_image) = image_url.split_once('/').unwrap();
        let params = CreateImageOptionsBuilder::new()
            .from_image(repo_image)
            .tag(tag)
            .build();
        let art = self.artifactory.clone().unwrap();
        let creds = DockerCredentials {
            username: art.username,
            password: art.password,
            serveraddress: Some(art.server),
            ..Default::default()
        };
        let docker = self.connection.clone();
        let mut stream = docker.create_image(Some(params), None, Some(creds));

        while let Some(info) = stream.try_next().await? {
            println!("{:?}", info.status);
            if let Some(err) = &info.error {
                eprintln!("Image pull failed: {}", err);
                return Err(Box::<dyn std::error::Error>::from(err.clone()));
            }
        }
        Ok(repo_image.to_string())
    }
}

#[async_trait::async_trait]
impl Remote for DockerProvider {}

#[async_trait::async_trait]
impl DevBoxProvider for DockerProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create_devbox(
        &mut self,
        builder: &str,
        devcontainer: DevBox,
        path: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let docker = self.connection.clone();

        let mut img = devcontainer
            .create_image(builder, path.clone(), self.artifactory.clone())
            .await?;
        if self.artifactory.is_some() {
            img = self.pull_image(&img, "latest").await?;
        }
        sleep(Duration::from_secs(10)).await;
        let id = create(docker.clone(), &devcontainer, &path, img).await?;
        if devcontainer.start_on_create.unwrap_or(false) {
            start(docker.clone(), &id).await?;
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
        Ok(id)
    }

    async fn delete_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let docker = self.connection.clone();
        match remove(docker, id).await {
            Ok(()) => (),
            _ => return Err("Fail to delete container".into()),
        };
        Ok(())
    }

    async fn start_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let docker = self.connection.clone();
        match start(docker, id).await {
            Ok(()) => (),
            _ => return Err("Fail to start container".into()),
        }
        Ok(())
    }

    async fn stop_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let docker = self.connection.clone();
        match stop(docker, id).await {
            Ok(()) => (),
            _ => return Err("Fail to stop container".into()),
        }
        Ok(())
    }

    async fn get_status(&self, id: String) -> Result<String, Box<dyn std::error::Error>> {
        let docker = self.connection.clone();
        status(docker, id).await
    }
}
