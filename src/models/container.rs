use crate::utils::build_from_local;
use bollard::Docker;
use bollard::models::ContainerCreateBody;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Default, serde::Serialize, serde::Deserialize, Clone, ToSchema)]
pub struct ContainerConfig {
    pub image: String,
    pub features: Option<Vec<String>>,
}

#[derive(Default, Serialize, Deserialize, Clone, ToSchema)]
pub struct Container {
    pub name: String,
    pub status: Option<String>,
    pub config: ContainerConfig,
}

impl Container {
    pub fn new(name: &str, status: Option<&str>, container: Container) -> Self {
        Self {
            name: name.to_string(),
            status: match status {
                Some(val) => Some(val.to_string()),
                _ => Some("Creating".to_string()),
            },
            ..container
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DevBox {
    pub name: String,
    pub image: String,
    pub build: Option<Build>,
    pub post_start_script: Option<Vec<String>>,
    pub start_on_create: Option<bool>,
    pub features: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    // pub config: Option<ContainerCreateBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Build {
    pub dockerfile: String,
    pub context: Option<String>,
}

impl DevBox {
    pub fn is_valid(&self) -> bool {
        self.build.is_some() || !self.image.is_empty()
    }

    pub async fn new(path: String) -> Self {
        let devbox_cfg_path = format!("{}/devbox.json", path.trim_end_matches('/'));
        let devbox_json = std::fs::read_to_string(&devbox_cfg_path).unwrap();
        println!("Devbox JSON: {}", devbox_json);
        let devbox: DevBox = serde_json::from_str(&devbox_json).unwrap();
        devbox
    }

    pub async fn create_image(
        &self,
        path: String,
        docker: Arc<Docker>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // let devbox_cfg_path = format!("{}/devbox.json", path.trim_end_matches('/'));
        // println!("Devbox Config Path: {}", devbox_cfg_path);
        // let devbox_json = std::fs::read_to_string(&devbox_cfg_path)?;
        // println!("Devbox JSON: {}", devbox_json);
        // let devbox: DevBox = serde_json::from_str(&devbox_json)?;
        build_from_local(docker, self, Some(&path.trim_end_matches('/').to_string())).await?;

        Ok(())
    }
}
