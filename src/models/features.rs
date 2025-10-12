use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevcontainerFeature {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub documentation_url: Option<String>,
    pub license_url: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub options: Option<BTreeMap<String, FeatureOption>>,
    pub container_env: Option<BTreeMap<String, String>>,
    pub privileged: Option<bool>,
    pub init: Option<bool>,
    pub cap_add: Option<Vec<String>>,
    pub security_opt: Option<Vec<String>>,
    pub entrypoint: Option<String>,
    pub customizations: Option<BTreeMap<String, serde_json::Value>>,
    pub depends_on: Option<BTreeMap<String, serde_json::Value>>,
    pub installs_after: Option<Vec<String>>,
    pub legacy_ids: Option<Vec<String>>,
    pub deprecated: Option<bool>,
    pub mounts: Option<Vec<Mount>>,
    // Lifecycle hooks
    pub on_create_command: Option<LifecycleCommand>,
    pub update_content_command: Option<LifecycleCommand>,
    pub post_create_command: Option<LifecycleCommand>,
    pub post_start_command: Option<LifecycleCommand>,
    pub post_attach_command: Option<LifecycleCommand>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureOption {
    #[serde(rename = "type")]
    pub option_type: String, // "string" or "boolean"
    pub description: Option<String>,
    pub proposals: Option<Vec<String>>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LifecycleCommand {
    Single(String),
    Multiple(Vec<String>),
    ParallelGroup(BTreeMap<String, String>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mount {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub mount_type: String,
}
