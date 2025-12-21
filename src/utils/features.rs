use crate::models::{DevBox, DevcontainerFeature, FeatureNode};
use crate::task_log;
use bollard::Docker;
use http_body_util::Full;
use parse_dockerfile::{Instruction, parse};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio_stream::StreamExt;

async fn get_docker_file(
    devcontainer: &DevBox,
    sorted_features: Option<&Vec<String>>,
    dev_path: &str,
) -> Result<(String, Option<Vec<String>>, Option<Vec<String>>), Box<dyn std::error::Error>> {
    let sorted_dev_features = sorted_features
        .as_ref()
        .map(|features| features.join("\n"))
        .unwrap_or_default();

    let base_image = format!("FROM {}\n{}", &devcontainer.image, sorted_dev_features);

    let (dockerfile, sourcefile, sourcedir) = match &devcontainer.build {
        Some(build) => {
            task_log!("Got build, reading Dockerfile from path...");
            task_log!("Features will be applied on top of dockerfile base image.");
            let context = build.context.as_deref().unwrap_or(".");

            // Determine dockerfile path (declare outside branches so it's in scope)
            let docker_file_path = if context.starts_with("/") {
                task_log!("Absolute context paths provided.");
                format!("{}/{}", context.trim_end_matches('/'), build.dockerfile)
            } else {
                format!(
                    "{}/../{}/{}",
                    dev_path.trim_end_matches('/'),
                    context.trim_end_matches('/'),
                    build.dockerfile
                )
            };

            let org_dockerfile = std::fs::read_to_string(&docker_file_path)?;
            // Remove the first line from the original Dockerfile before concatenating
            let org_dockerfile_without_first = org_dockerfile
                .lines()
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n");

            let dockerfile = format!("{}\n{}", base_image, org_dockerfile_without_first);

            // Get all the copy directories
            let dockerfile_parser = parse(&org_dockerfile).unwrap();
            let mut sources_files: Vec<String> = Vec::new();
            let mut sources_dirs: Vec<String> = Vec::new();

            for inst in dockerfile_parser.instructions {
                if let Instruction::Copy(copy) = inst {
                    for src in &copy.src {
                        if let parse_dockerfile::Source::Path(unescaped) = src {
                            let src_str = unescaped.value.as_ref();
                            let path = Path::new(src_str);

                            let source_path = if path.is_absolute() {
                                path.to_string_lossy().to_string()
                            } else {
                                let base_path = Path::new(&docker_file_path).parent().unwrap();
                                format!("{}/{}", base_path.display(), src_str)
                            };

                            let resolved_path = Path::new(&source_path);
                            if resolved_path.is_file() {
                                sources_files.push(source_path);
                            } else if resolved_path.is_dir() {
                                sources_dirs.push(source_path);
                            }
                        }
                    }
                }
            }

            (dockerfile, Some(sources_files), Some(sources_dirs))
        }
        None => {
            task_log!("No build specified, generating Dockerfile from base image and features...");
            (base_image, None, None)
        }
    };

    Ok((dockerfile, sourcefile, sourcedir))
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

    for option_name in option_json.keys() {
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
    task_log!("FULL: {full_path}");
    let feature_json = std::fs::read_to_string(&full_path)?;
    let feature: DevcontainerFeature = serde_json::from_str(&feature_json)?;
    task_log!("Loading feature: {}", feature.id);
    let _ = process_feature(feature.clone(), &params, features_cmd_map);

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
    task_log!("Generating Dockerfile from features path: {}", base_path);
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
    task_log!("Resolved Feature Install Order:");
    let reversed: Vec<_> = feature_order.iter().rev().cloned().collect();
    for node in &reversed {
        for id in node.feature.keys() {
            let ky = id.rsplit('/').next().unwrap_or("");
            if let Some(cmds) = features_cmd_map.get(ky) {
                docker_file_features.extend_from_slice(cmds);
                features_cmd_map.remove(ky);
            }
        }
    }
    task_log!("Dockerfile so far: {:?}", docker_file_features);

    Ok(docker_file_features)
}

pub async fn build_from_local(
    docker: Arc<Docker>,
    devcontainer: &DevBox,
    path: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    // let curr_path = ".".to_string();
    // let base_path = path.unwrap_or(&curr_path);
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
    let mut image_build_stream = docker.build_image(
        build_image_options.build(),
        None,
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
