use bollard::models::ContainerCreateBody;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct ContainerConfig {
    pub image: String,
    pub features: Option<Vec<String>>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevBox {
    pub name: String,
    pub image: Option<String>,
    pub build: Option<Build>,
    pub post_start_script: Option<Vec<String>>,
    pub start_on_create: Option<bool>,
    pub features: Option<HashMap<String, Feature>>,
    pub config: Option<ContainerCreateBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub dockerfile: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub version: Option<String>,
}

impl DevBox {
    pub fn is_valid(&self) -> bool {
        self.image.as_ref().map(|s| !s.is_empty()).unwrap_or(false) || self.build.is_some()
    }
}
