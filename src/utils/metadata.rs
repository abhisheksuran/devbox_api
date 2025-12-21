use crate::models::{DevBox, DevcontainerFeature, FeatureNode};
use crate::task_log;
use std::collections::HashMap;

#[derive(Default)]
pub struct MergedMetadata {
    pub init: bool,
    pub privileged: bool,
    pub cap_add: Vec<String>,
    pub security_opt: Vec<String>,
    pub entrypoints: Vec<String>,
    pub mounts: HashMap<String, serde_json::Value>, // dst -> mount object
    on_create_commands: Vec<serde_json::Value>,
    update_content_commands: Vec<serde_json::Value>,
    post_create_commands: Vec<serde_json::Value>,
    post_start_commands: Vec<serde_json::Value>,
    post_attach_commands: Vec<serde_json::Value>,
    container_env: HashMap<String, String>,
}

impl MergedMetadata {
    fn merge_feature(
        &mut self,
        feature: DevcontainerFeature,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.init |= feature.init.unwrap_or(false);
        self.privileged |= feature.privileged.unwrap_or(false);

        if let Some(caps) = feature.cap_add {
            self.cap_add.extend(caps);
            self.cap_add.sort();
            self.cap_add.dedup();
        }

        if let Some(sec) = feature.security_opt {
            self.security_opt.extend(sec);
            self.security_opt.sort();
            self.security_opt.dedup();
        }

        if let Some(ep) = feature.entrypoint {
            // If entrypoint is Vec<String>
            self.entrypoints.push(ep);
            // If entrypoint is vec, use: self.entrypoints.extend(ep);
        }

        if let Some(mounts) = feature.mounts {
            for m in mounts {
                // Assuming Mount struct with dst field
                self.mounts
                    .insert(m.target.clone(), serde_json::to_value(m)?);
            }
        }

        if let Some(cmds) = feature.on_create_command {
            self.on_create_commands.push(serde_json::to_value(cmds)?);
        }
        if let Some(cmds) = feature.update_content_command {
            self.update_content_commands
                .push(serde_json::to_value(cmds)?);
        }
        if let Some(cmds) = feature.post_create_command {
            self.post_create_commands.push(serde_json::to_value(cmds)?);
        }
        if let Some(cmds) = feature.post_start_command {
            self.post_start_commands.push(serde_json::to_value(cmds)?);
        }
        if let Some(cmds) = feature.post_attach_command {
            self.post_attach_commands.push(serde_json::to_value(cmds)?);
        }

        if let Some(env) = feature.container_env {
            for (k, v) in env {
                self.container_env.insert(k, v);
            }
        }

        Ok(())
    }
}

fn load_feature(
    name: String,
    params: HashMap<String, serde_json::Value>,
    base_path: &String,
    feature_order: &mut Vec<FeatureNode>,
    required: bool,
    metadata: &mut MergedMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let full_path = format!("{}/features/{}/devcontainer-feature.json", base_path, name);
    task_log!("FULL: {full_path}");
    let feature_json = std::fs::read_to_string(&full_path)?;
    let feature: DevcontainerFeature = serde_json::from_str(&feature_json)?;
    task_log!("Loading feature: {}", feature.id);

    let _ = metadata.merge_feature(feature.clone());

    // Add current feature
    let mut feature_map = HashMap::new();
    feature_map.insert(name.clone(), params.clone());
    feature_order.push(FeatureNode {
        feature: feature_map,
        required,
    });

    // Recursively load dependencies
    if let Some(depends_on) = feature.depends_on {
        for (dep_path, dep_config) in depends_on {
            load_feature(
                dep_path.clone(),
                dep_config.clone(),
                base_path,
                feature_order,
                true,
                metadata,
            )?;
        }
    }

    if let Some(installs_after) = feature.installs_after {
        for dep_path in installs_after {
            load_feature(
                dep_path,
                HashMap::new(),
                base_path,
                feature_order,
                false,
                metadata,
            )?;
        }
    }

    Ok(())
}

pub fn feature_runtime(
    base_path: &String,
    devbox: &DevBox,
) -> Result<MergedMetadata, Box<dyn std::error::Error>> {
    task_log!("Generating metadata from features path: {}", base_path);
    let mut feature_order: Vec<FeatureNode> = Vec::new();
    let mut metadata = MergedMetadata::default();

    if let Some(features) = &devbox.features {
        for (path, config_map) in features {
            load_feature(
                path.clone(),
                config_map.clone(),
                base_path,
                &mut feature_order,
                true,
                &mut metadata,
            )?;
        }
    }

    Ok(metadata)
}
