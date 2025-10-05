use bollard::Docker;
use futures_util::TryStreamExt;
use futures_util::stream::StreamExt;
use http_body_util::Full;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use tracing::{error, info};

use crate::models::DevBox;

pub async fn create_image(
    docker: Arc<Docker>,
    devcontainer: &DevBox,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(image_name) = &devcontainer.image {
        if !image_name.is_empty() {
            println!("Pulling image: {}", image_name);

            let options = bollard::query_parameters::CreateImageOptionsBuilder::default()
                .from_image(&image_name)
                .tag("latest")
                .repo(&devcontainer.name)
                .build();
            docker
                .create_image(Some(options), None, None)
                .try_collect::<Vec<_>>()
                .await?;
            println!("Tagging image..");
            let tag_options = bollard::query_parameters::TagImageOptions {
                repo: Some(devcontainer.name.clone()),
                tag: Some("latest".to_string()),
            };

            docker
                .tag_image(&format!("{}:latest", image_name), Some(tag_options))
                .await?;
        } else {
            // return Err(Box::new(std::io::Error::other("Image is null")));
            build_from_local(docker, devcontainer).await?;
        }
    }
    Ok(())
}

pub async fn build_from_remote(
    docker: Arc<Docker>,
    devcontainer: &DevBox,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building using docker file");
    let mut build_image_args = HashMap::new();
    build_image_args.insert("dummy", "value");

    let mut build_image_labels = HashMap::new();
    build_image_labels.insert("maintainer", "somemaintainer");

    let build_image_options = bollard::query_parameters::BuildImageOptionsBuilder::default()
        .dockerfile("Dockerfile")
        .t("latest")
        .remote("remote url for docker file")
        .extrahosts("myhost:127.0.0.1")
        .q(true)
        .nocache(false)
        .pull("true")
        .rm(true)
        .forcerm(true)
        .memory(120000000)
        .memswap(500000)
        .cpushares(2)
        .cpusetcpus("0-3")
        .cpuperiod(2000)
        .cpuquota(1000)
        .buildargs(&build_image_args)
        .shmsize(1000000)
        .squash(false)
        .labels(&build_image_labels)
        .networkmode("host")
        .platform("linux/x86_64");

    let builder = build_image_options.build();
    let mut image_build_stream = docker.build_image(builder, None, None);
    while let Some(msg) = image_build_stream.next().await {
        println!("Message: {msg:?}");
    }
    Ok(())
}

pub async fn build_from_local(
    docker: Arc<Docker>,
    devcontainer: &DevBox,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = devcontainer
        .build
        .as_ref()
        .unwrap()
        .context
        .as_deref()
        .unwrap_or(".");
    let docker_file_path = format!(
        "{}/{}",
        context.trim_end_matches('/'),
        devcontainer.build.as_ref().unwrap().dockerfile
    );
    let dockerfile = std::fs::read_to_string(docker_file_path)?;

    println!("{:?}", &dockerfile);

    let mut header = tar::Header::new_gnu();
    header.set_path("Dockerfile").unwrap();
    header.set_size(dockerfile.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    let mut tar = tar::Builder::new(Vec::new());
    tar.append(&header, dockerfile.as_bytes()).unwrap();

    let uncompressed = tar.into_inner().unwrap();
    let mut c = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    c.write_all(&uncompressed).unwrap();
    let compressed = c.finish().unwrap();

    let id = &devcontainer.name;
    let build_image_options = bollard::query_parameters::BuildImageOptionsBuilder::default()
        .t(id)
        .dockerfile("Dockerfile")
        .pull("true");

    println!("Building image..");
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
                    error!("Build error: {}", error);
                }
            }
            Err(e) => {
                error!("Stream error: {:?}", e);
            }
        }
    }
    Ok(())
}
