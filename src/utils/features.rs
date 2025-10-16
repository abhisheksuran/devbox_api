use crate::models::{Build, DevBox, DevcontainerFeature, FeatureNode};
use bollard::Docker;
use http_body_util::Full;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::info;

async fn get_docker_file(
    devcontainer: &DevBox,
    sorted_features: Option<&Vec<String>>,
    dev_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let sorted_dev_features = sorted_features
        .as_ref()
        .map(|features| features.join("\n"))
        .unwrap_or_default();

    let dockerfile = match &devcontainer.features {
        Some(_features) => {
            info!("Got features, generating Dockerfile with features...");
            format!("FROM {}\n{}", &devcontainer.image, sorted_dev_features)
        }
        None => {
            info!("No features found, using base image Dockerfile or defaulting to FROM debian");
            // let curr_pth = ".".to_string();
            // let pth = dev_path.unwrap_or(&curr_pth);
            let context = devcontainer
                .build
                .as_ref()
                .unwrap()
                .context
                .as_deref()
                .unwrap_or(dev_path);
            let docker_file_path = format!(
                "{}/{}",
                context.trim_end_matches('/'),
                devcontainer.build.as_ref().unwrap().dockerfile
            );
            std::fs::read_to_string(docker_file_path)?
        }
    };
    Ok(dockerfile)
}

fn process_feature(
    feature: DevcontainerFeature,
    params: &HashMap<String, serde_json::Value>,
    features_cmd_map: &mut HashMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut run_commands: Vec<String> = Vec::new();
    let option_json = feature.options.unwrap_or_default();
    let container_env = feature.container_env.unwrap_or_default();

    for (option_name, option_data) in &option_json {
        let value_str = match params.get(option_name.as_str()) {
            Some(serde_json::Value::String(s)) => {
                format!("ENV {}={}", option_name.to_uppercase(), s)
            }
            Some(serde_json::Value::Bool(b)) => format!("ENV {}={}", option_name.to_uppercase(), b),
            Some(other) => format!("ENV {}={}", option_name.to_uppercase(), other),
            None => format!(
                "ENV {}={}",
                option_name.to_uppercase(),
                option_data.default.clone().unwrap_or_default()
            ),
        };

        run_commands.push(value_str);
    }

    for (key, value) in &container_env {
        run_commands.push(format!("ENV {}={}", key.to_uppercase(), value));
    }

    run_commands.push(format!(
        "COPY ./features/{} /features/{}",
        feature.id, feature.id
    ));
    run_commands.push(format!(
        "RUN cd /features/{}/ && bash ./install.sh",
        feature.id.rsplit('/').next().unwrap_or("")
    ));

    for (option_name, _option_data) in &option_json {
        run_commands.push(format!("ENV {}=''", option_name.to_uppercase()));
    }

    features_cmd_map.insert(feature.id.clone(), run_commands);
    Ok(())
}

// Use a mutable vector passed around instead of static
fn load_feature(
    name: String,
    params: HashMap<String, serde_json::Value>,
    base_path: &String,
    feature_order: &mut Vec<FeatureNode>,
    required: bool,
    features_cmd_map: &mut HashMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let full_path = format!("{}/features/{}/devcontainer-feature.json", base_path, name);
    info!("FULL: {full_path}");
    let feature_json = std::fs::read_to_string(&full_path)?;
    let feature: DevcontainerFeature = serde_json::from_str(&feature_json)?;
    info!("Loading feature: {}", feature.id);
    let _ = process_feature(feature.clone(), &params, features_cmd_map);

    // Add current feature
    let mut feature_map = HashMap::new();
    feature_map.insert(name.clone(), params.clone());
    feature_order.push(FeatureNode {
        feature: feature_map,
        required: required,
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
                features_cmd_map,
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
                features_cmd_map,
            )?;
        }
    }

    Ok(())
}

fn feature_to_dockerfile(
    base_path: &String,
    devbox: &DevBox,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    info!("Generating Dockerfile from features path: {}", base_path);
    let mut feature_order: Vec<FeatureNode> = Vec::new();
    let mut features_cmd_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut docker_file_features: Vec<String> = Vec::new();
    docker_file_features.push("RUN useradd -m -d /home/vscode vscode".to_string());
    docker_file_features.push("RUN touch home/vscode/.zshrc".to_string());
    docker_file_features.push("RUN mkdir /features".to_string());

    if let Some(features) = &devbox.features {
        for (path, config_map) in features {
            load_feature(
                path.clone(),
                config_map.clone(),
                base_path,
                &mut feature_order,
                true,
                &mut features_cmd_map,
            )?;
        }
    }
    // info!("Features Command Map: {:?}", features_cmd_map);
    info!("Resolved Feature Install Order:");
    let reversed: Vec<_> = feature_order.iter().rev().cloned().collect();
    for node in &reversed {
        for (id, _config) in &node.feature {
            let ky = id.rsplit('/').next().unwrap_or("");
            if let Some(cmds) = features_cmd_map.get(ky) {
                docker_file_features.extend_from_slice(cmds);
                features_cmd_map.remove(ky);
            }
        }
    }
    info!("Dockerfile so far: {:?}", docker_file_features);

    Ok(docker_file_features)
}

pub async fn build_from_local(
    docker: Arc<Docker>,
    devcontainer: &DevBox,
    path: Option<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let curr_path = ".".to_string();
    let base_path = path.unwrap_or(&curr_path);
    let sorted_dev_features = feature_to_dockerfile(base_path, devcontainer).unwrap_or_default();
    info!("Final Dockerfile Commands: {:?}", sorted_dev_features);
    let dockerfile = get_docker_file(
        &devcontainer,
        Some(&sorted_dev_features),
        base_path.as_str(),
    )
    .await?;

    info!("{:?}", &dockerfile);

    let mut header = tar::Header::new_gnu();
    header.set_path("Dockerfile").unwrap();
    header.set_size(dockerfile.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    let mut tar = tar::Builder::new(Vec::new());
    tar.append(&header, dockerfile.as_bytes()).unwrap();

    let feature_dir = format!("{}/features", path.unwrap_or(&".".to_string()));
    let feature_path = std::path::Path::new(&feature_dir);
    tar.append_dir_all("features", feature_path)?;

    let uncompressed = tar.into_inner().unwrap();
    let mut c = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    c.write_all(&uncompressed).unwrap();
    let compressed = c.finish().unwrap();

    let id = &devcontainer.name;
    let build_image_options = bollard::query_parameters::BuildImageOptionsBuilder::default()
        .t(id)
        .dockerfile("Dockerfile")
        .pull("true");

    info!("Building image..");
    let mut image_build_stream = docker.build_image(
        build_image_options.build(),
        None,
        Some(http_body_util::Either::Left(Full::new(compressed.into()))),
    );

    while let Some(msg) = image_build_stream.next().await {
        match msg {
            Ok(info) => {
                if let Some(stream) = info.stream {
                    info!("Stream {}", stream);
                }
                if let Some(status) = info.status {
                    info!("Status: {}", status);
                }
                if let Some(aux) = info.aux {
                    info!("Image ID: {:?}", aux.id);
                }
                if let Some(error) = info.error {
                    info!("Build error: {}", error);
                }
            }
            Err(e) => {
                info!("Stream error: {:?}", e);
            }
        }
    }
    Ok(())
}
