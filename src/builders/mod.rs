pub mod docker;
use crate::apps::artifactory::Artifactory;
use crate::db::get_builder;
use crate::models::DevBox;

#[async_trait::async_trait]
pub trait Builder {
    async fn init(
        name: &str,
        artifactory: Option<Artifactory>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    async fn build(
        &self,
        devcontainer: &DevBox,
        path: &String,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn push(
        &mut self,
        devbox_image: &str,
        tag: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

pub async fn get_builder_strategy(
    name: &str,
    artifactory: Option<Artifactory>,
) -> Result<Box<dyn Builder + Send>, Box<dyn std::error::Error>> {
    let builder_data = get_builder(name).await?;
    let builder_app = builder_data
        .get("builder")
        .and_then(serde_json::Value::as_str)
        .unwrap();
    match builder_app {
        "docker" => {
            let builder = docker::DockerBuilder::init(name, artifactory).await?;
            Ok(Box::new(builder))
        }
        _ => Err(Box::new(std::io::Error::other("No valid builder provided"))),
    }
}
