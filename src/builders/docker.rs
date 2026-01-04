use crate::builders::Builder;
use crate::db::get_builder;
use crate::models::DevBox;
use crate::task_log;
use crate::utils::Remote;
use crate::utils::TunnelConfig;
use crate::utils::artifactory::Artifactory;
use crate::utils::{feature_to_dockerfile, get_docker_file};
use bollard::auth::DockerCredentials;
use bollard::query_parameters::PushImageOptions;
use bollard::query_parameters::TagImageOptions;
use http_body_util::Full;
use russh::client::Handle;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::info;

type BuilderTunnel = Option<(
    Option<tokio::task::JoinHandle<()>>,
    Option<CancellationToken>,
    Option<Arc<Handle<crate::utils::Client>>>,
)>;
pub struct DockerBuilder {
    pub connection: bollard::Docker,
    pub config: Option<TunnelConfig>,
    pub artifactory: Option<Artifactory>,
    pub tunnel: BuilderTunnel,
}

#[async_trait::async_trait]
impl Remote for DockerBuilder {}

impl DockerBuilder {
    async fn get_auth(&self) -> Option<DockerCredentials> {
        match self.artifactory.clone() {
            Some(art) => Some(DockerCredentials {
                username: art.username,
                password: art.password,
                auth: None,
                email: None,
                serveraddress: Some(art.server),
                identitytoken: None,
                registrytoken: None,
            }),
            None => None,
        }
    }
    async fn get_auth_hash(&self) -> Option<HashMap<String, DockerCredentials>> {
        match self.artifactory.clone() {
            Some(art) => {
                let mut creds_map = HashMap::new();
                creds_map.insert(
                    art.server.clone(),
                    DockerCredentials {
                        username: art.username,
                        password: art.password,
                        serveraddress: Some(art.server),
                        ..Default::default()
                    },
                );
                Some(creds_map)
            }
            None => None,
        }
    }

    pub async fn stop_tunnel(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some((mut handle, cancel, session)) = self.tunnel.take() {
            cancel.unwrap().cancel();
            handle.unwrap().await?;
            DockerBuilder::disconnect(session.unwrap()).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Builder for DockerBuilder {
    async fn init(
        name: &str,
        artifactory: Option<Artifactory>,
    ) -> Result<DockerBuilder, Box<dyn std::error::Error>> {
        let builder_data = get_builder(name).await?;
        let builder = builder_data
            .get("builder")
            .and_then(serde_json::Value::as_str)
            .unwrap();
        if builder != "docker" {
            return Err(Box::new(std::io::Error::other(
                "Error initializing builder",
            )));
        }
        let connection: bollard::Docker;
        if builder_data
            .get("remote")
            .and_then(serde_json::Value::as_bool)
            .unwrap()
        {
            let builder_cfg: TunnelConfig = builder_data
                .get("config")
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        serde_json::from_str(s).unwrap()
                    } else {
                        serde_json::from_value(v.clone()).unwrap()
                    }
                })
                .unwrap();
            let mut cancel: Option<CancellationToken> = None;
            let mut handel: Option<tokio::task::JoinHandle<()>> = None;
            let mut ssh_session: Option<Arc<Handle<crate::utils::Client>>> = None;
            if !DockerBuilder::check_port("127.0.0.1", builder_cfg.local_port as u16, None).await {
                ssh_session = Some(Arc::new(DockerBuilder::connect(builder_cfg.clone()).await?));

                cancel = Some(CancellationToken::new());
                handel = Some(tokio::spawn(DockerBuilder::run_tunnel(
                    ssh_session.clone().unwrap(),
                    builder_cfg.service_port,
                    builder_cfg.local_port,
                    cancel.clone().unwrap(),
                )));
            }

            let docker_con = bollard::Docker::connect_with_http(
                format!("127.0.0.1:{}", builder_cfg.local_port).as_str(),
                10,
                bollard::API_DEFAULT_VERSION,
            )
            .unwrap();
            info!("Using Rmote Docker Builder");
            Ok(DockerBuilder {
                connection: docker_con,
                config: Some(builder_cfg),
                artifactory,
                tunnel: Some((handel, cancel, ssh_session)),
            })
        } else {
            info!("Using Local Docker Builder");
            let docker_local = bollard::Docker::connect_with_defaults().unwrap();
            Ok(DockerBuilder {
                connection: docker_local,
                config: None,
                artifactory,
                tunnel: None,
            })
        }
    }

    async fn build(
        &self,
        devcontainer: &DevBox,
        path: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sorted_dev_features = feature_to_dockerfile(path, devcontainer).unwrap_or_default();
        task_log!("Final Dockerfile Commands: {:?}", sorted_dev_features);
        let (dockerfile, sourcefile, sourcedir) =
            get_docker_file(devcontainer, Some(&sorted_dev_features), path.as_str()).await?;

        task_log!("{:?}", &dockerfile);

        let mut header = tar::Header::new_gnu();
        header.set_path("Dockerfile").unwrap();
        header.set_size(dockerfile.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        let mut tar = tar::Builder::new(Vec::new());
        tar.append(&header, dockerfile.as_bytes()).unwrap();

        let feature_dir = format!("{}/features", path);
        let feature_path = std::path::Path::new(&feature_dir);
        tar.append_dir_all("features", feature_path)?;

        // Add source files
        if let Some(files) = sourcefile {
            for file_path in files {
                let file_path_obj = Path::new(&file_path);
                if file_path_obj.is_file() {
                    tar.append_path_with_name(file_path_obj, file_path_obj.file_name().unwrap())?;
                }
            }
        }

        // Add source directories
        if let Some(dirs) = sourcedir {
            for dir_path in dirs {
                let dir_path_obj = Path::new(&dir_path);
                if dir_path_obj.is_dir() {
                    tar.append_dir_all(dir_path_obj.file_name().unwrap(), dir_path_obj)?;
                }
            }
        }

        let uncompressed = tar.into_inner().unwrap();
        let mut c = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        c.write_all(&uncompressed).unwrap();
        let compressed = c.finish().unwrap();

        let id = &devcontainer.name;
        let build_image_options = bollard::query_parameters::BuildImageOptionsBuilder::default()
            .t(id)
            .dockerfile("Dockerfile")
            .pull("true");

        task_log!("Building image..");
        let mut image_build_stream = self.connection.build_image(
            build_image_options.build(),
            self.get_auth_hash().await,
            Some(http_body_util::Either::Left(Full::new(compressed.into()))),
        );

        while let Some(msg) = image_build_stream.next().await {
            match msg {
                Ok(info) => {
                    if let Some(stream) = info.stream {
                        task_log!("Stream {}", stream);
                    }
                    if let Some(status) = info.status {
                        task_log!("Status: {}", status);
                    }
                    if let Some(aux) = info.aux {
                        task_log!("Image ID: {:?}", aux.id);
                    }
                    if let Some(error) = info.error {
                        task_log!("Build error: {}", error);
                    }
                }
                Err(e) => {
                    task_log!("Stream error: {:?}", e);
                }
            }
        }
        Ok(())
    }

    async fn push(
        &mut self,
        devbox_image: &str,
        tag: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Initialize Docker client
        let docker = &self.connection;
        let artifactory = self.artifactory.clone().unwrap();
        // Define the image name and tag
        let image_name = format!(
            "{}/{}/{}",
            artifactory.server, artifactory.repository_name, devbox_image
        );

        task_log!("Tagging image");
        let tag_options = TagImageOptions {
            repo: Some(image_name.clone()),
            tag: Some("latest".to_string()),
        };

        match docker.tag_image(devbox_image, Some(tag_options)).await {
            Ok(_) => task_log!("Image tagged successfully"),
            Err(e) => task_log!("Failed to tag image: {}", e),
        }
        task_log!("Tagging completed");
        // Set up authentication credentials for Artifactory
        let credentials = self.get_auth().await;

        // Push image options
        let push_options = PushImageOptions {
            tag: Some(tag.to_string()),
            ..Default::default()
        };

        // Push the image
        let mut stream = docker.push_image(&image_name, Some(push_options), credentials);

        // Stream the output
        while let Some(output) = stream.next().await {
            match output {
                Ok(log) => task_log!("{:?}", log),
                Err(e) => task_log!("Error: {}", e),
            }
        }
        self.stop_tunnel().await;
        Ok(image_name)
    }
}
