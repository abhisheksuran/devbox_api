use crate::db::DefaultDB;
use crate::models::{Container, ContainerConfig, DevBox};
use crate::providers::ProviderEnum;
use crate::providers::docker::image::create_image;
use crate::providers::docker::{
    create as docker_create, exec as docker_exec, remove as docker_remove, start as docker_start,
};
use bollard::Docker;
use futures::future::ok;
use rusqlite::params;
use std::sync::Arc;
use tracing::{error, info};

pub async fn list_containers() -> Result<String, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;

    let mut stmt = conn.prepare("SELECT name, status, config FROM containers")?;
    let container_iter = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let status: String = row.get(1)?;
        let config_str: String = row.get(2)?;
        let config: ContainerConfig = serde_json::from_str(&config_str).unwrap();
        Ok(Container {
            name,
            status: Some(status),
            config,
        })
    })?;

    let containers: Result<Vec<Container>, _> = container_iter.collect();
    let json = serde_json::to_string(&containers?)?;
    Ok(json)
}

pub async fn create_container(input: Container) -> Result<(), Box<dyn std::error::Error>> {
    let ipt = Container::new(&input.name, input.status.as_deref(), input.clone());
    let config_json = serde_json::to_string(&input.config)?;
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "INSERT INTO containers (name, status, config) VALUES (?1, ?2, ?3)",
        [&ipt.name, &ipt.status.unwrap(), &config_json],
    )?;
    Ok(())
}

pub async fn fetch_container(name: String) -> Result<String, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    let mut stmt = conn.prepare("SELECT name, status, config FROM containers WHERE name = ?1")?;

    let container = stmt.query_row(params![name], |row| {
        let json_str: String = row.get(2)?;
        Ok(Container {
            name: row.get(0)?,
            status: row.get(1)?,
            config: serde_json::from_str(&json_str).unwrap(),
        })
    })?;
    let json = serde_json::to_string(&container)?;
    Ok(json)
}

// pub async fn create_devbox(
//     provider: ProviderEnum,
//     docker: Arc<Docker>,
//     devcontainer: DevBox,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     match provider {
//         ProviderEnum::DOCKER => {
//             info!("Creating image");
//             create_image(docker.clone(), &devcontainer).await?;
//             let id = docker_create(docker.clone(), &devcontainer).await?;
//             let flag = devcontainer.start_on_create.unwrap_or_default();
//             if flag {
//                 docker_start(docker.clone(), id.clone()).await?;
//                 info!("Container started.");
//             }
//             if let Some(command) = devcontainer.post_start_script {
//                 docker_exec(docker, id.clone(), Some(command)).await?
//             }
//             Ok(id)
//         }
//         ProviderEnum::AZURE => Ok("dummy_id".to_string()),
//         ProviderEnum::AWS => Ok("dummy_id".to_string()),
//     }
// }
use axum::extract::ws::Utf8Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
pub async fn create_devbox(
    stream: WebSocket,
    provider: ProviderEnum,
    docker: Arc<Docker>,
    // devcontainer: DevBox,
) {
    info!("inside create devbox");
    let devcontainer = DevBox {
        name: "new_ws".to_string(),
        image: Some("nginx".to_string()),
        build: None,
        post_start_script: None,
        start_on_create: None,
        features: None,
        config: None,
    };
    let (mut sender, mut receiver) = stream.split();
    match provider {
        ProviderEnum::DOCKER => {
            info!("Creating image");
            let utf8 = Utf8Bytes::from("Creating image".to_string());
            let _ = sender.send(Message::Text(utf8)).await.is_err();
            let _ = create_image(docker.clone(), &devcontainer).await;
            let id = docker_create(docker.clone(), &devcontainer).await.unwrap();
            let flag = devcontainer.start_on_create.unwrap_or_default();
            if flag {
                let _ = docker_start(docker.clone(), id.clone()).await;
                info!("Container started.");
            }
            if let Some(command) = devcontainer.post_start_script {
                let _ = docker_exec(docker, id.clone(), Some(command)).await;
            }
        }
        ProviderEnum::AZURE => (),
        ProviderEnum::AWS => (),
    }
}
