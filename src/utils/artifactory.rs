use tracing::{error, info};

use crate::task_log;
use bollard::Docker;
use bollard::auth::DockerCredentials;
use bollard::query_parameters::PushImageOptions;
use bollard::query_parameters::TagImageOptions;
use futures_util::stream::StreamExt;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Artifactory {
    pub server: String,
    pub repository_name: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Artifactory {
    pub fn new(
        server: String,
        repository_name: String,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        Artifactory {
            server,
            repository_name,
            username,
            password,
        }
    }

    pub async fn push_image(&self, devbox_image: &str, tag: &str) -> String {
        // Initialize Docker client
        let docker = Docker::connect_with_local_defaults().unwrap();

        // Define the image name and tag
        let image_name = format!("{}/{}/{}", self.server, self.repository_name, devbox_image);

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
        let credentials = DockerCredentials {
            username: self.username.clone(),
            password: self.password.clone(),
            serveraddress: Some(self.server.clone()),
            ..Default::default()
        };

        // Push image options
        let push_options = PushImageOptions {
            tag: Some(tag.to_string()),
            ..Default::default()
        };

        // Push the image
        let mut stream = docker.push_image(&image_name, Some(push_options), Some(credentials));

        // Stream the output
        while let Some(output) = stream.next().await {
            match output {
                Ok(log) => task_log!("{:?}", log),
                Err(e) => task_log!("Error: {}", e),
            }
        }
        image_name
    }
}
