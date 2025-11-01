use crate::task_log;
use crate::utils::build_from_local;
use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DevBox {
    pub name: String,
    pub image: String,
    pub build: Option<Build>,
    pub post_start_script: Option<Vec<String>>,
    pub start_on_create: Option<bool>,
    pub features: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    pub mounts: Vec<String>,
    pub remote_user: String,
    pub ports: Option<Vec<u16>>,
    pub cpu_limit: Option<f64>,
    pub memory_limit: Option<f64>,
    pub target_platform: Option<String>,
    pub environment: Option<HashMap<String, String>>,
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
        task_log!("Devbox JSON: {}", devbox_json);
        let devbox: DevBox = serde_json::from_str(&devbox_json).unwrap();
        devbox
    }

    pub async fn create_image(
        &self,
        path: String,
        docker: Arc<Docker>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        build_from_local(docker, self, &path.trim_end_matches('/').to_string()).await?;

        Ok(())
    }
}
