use crate::logs::TASK_LOGGERS;
use crate::models::DevBox;
use crate::task_log;
use crate::utils::metadata::feature_runtime;
use bollard::Docker;
use bollard::container::LogOutput;
use bollard::models::ContainerCreateBody;
use bollard::models::HostConfig;
use bollard::query_parameters::InspectContainerOptions;
use bollard::query_parameters::LogsOptions;
use futures::stream::StreamExt;
use std::sync::Arc;
use tracing::info;

pub async fn create(
    docker: Arc<Docker>,
    devcontainer: &DevBox,
    path: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    task_log!("Creating container...");

    let runtimeops = feature_runtime(path, devcontainer)?;

    let host_config = HostConfig {
        init: Some(runtimeops.init),
        privileged: Some(runtimeops.privileged),
        cap_add: Some(runtimeops.cap_add),
        security_opt: Some(runtimeops.security_opt),
        ..Default::default()
    };

    let image_config = ContainerCreateBody {
        image: Some(devcontainer.name.clone()),
        tty: Some(true),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        open_stdin: Some(true),
        host_config: Some(host_config),
        ..Default::default()
    };

    let id = docker
        .create_container(
            None::<bollard::query_parameters::CreateContainerOptions>,
            image_config,
        )
        .await?
        .id;

    Ok(id)
}

pub async fn stop(docker: Arc<Docker>, id: String) -> Result<(), Box<dyn std::error::Error>> {
    task_log!("Stopping container...");
    docker
        .stop_container(&id, None::<bollard::query_parameters::StopContainerOptions>)
        .await?;
    Ok(())
}

pub async fn start(docker: Arc<Docker>, id: String) -> Result<(), Box<dyn std::error::Error>> {
    task_log!("Starting container...");
    docker
        .start_container(
            &id,
            None::<bollard::query_parameters::StartContainerOptions>,
        )
        .await?;
    Ok(())
}

pub async fn remove(docker: Arc<Docker>, id: String) -> Result<(), Box<dyn std::error::Error>> {
    match docker
        .remove_container(
            &id,
            Some(
                bollard::query_parameters::RemoveContainerOptionsBuilder::default()
                    .force(true)
                    .build(),
            ),
        )
        .await
    {
        Ok(()) => Ok(()),
        _ => Err("Fail to delete container".into()),
    }
}

pub async fn status(docker: Arc<Docker>, id: String) -> Result<String, Box<dyn std::error::Error>> {
    match docker
        .inspect_container(&id, None::<InspectContainerOptions>)
        .await
    {
        Ok(info) => {
            if let Some(state) = info.state {
                match state.status {
                    Some(bollard::secret::ContainerStateStatusEnum::RUNNING) => {
                        Ok("running".to_string())
                    }
                    Some(bollard::secret::ContainerStateStatusEnum::EXITED) => {
                        Ok("exited".to_string())
                    }
                    _ => Ok("unknown".to_string()),
                }
            } else {
                Ok("unknown".to_string())
            }
        }
        Err(_e) => Err("Unable to get container status".into()),
    }
}

pub async fn exec(
    docker: Arc<Docker>,
    id: String,
    command: Option<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let exec = docker
        .create_exec(
            &id,
            bollard::models::ExecConfig {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: command,
                ..Default::default()
            },
        )
        .await?
        .id;
    if let bollard::exec::StartExecResults::Attached { mut output, .. } =
        docker.start_exec(&exec, None).await?
    {
        while let Some(Ok(msg)) = output.next().await {
            info!("{msg}");
        }
        Ok(())
    } else {
        unreachable!();
    }
}

/// Attach to container logs and forward each chunk to the provided sender.
pub async fn attach_container_logs(
    docker: Arc<Docker>,
    id: String,
    task_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    task_log!("Attaching to container logs...");
    let mut logs_stream = docker.logs(
        &id,
        Some(LogsOptions {
            follow: true,
            stdout: true,
            stderr: true,
            tail: "all".to_string(),
            ..Default::default()
        }),
    );

    while let Some(Ok(output)) = logs_stream.next().await {
        let msg = match output {
            LogOutput::StdOut { message }
            | LogOutput::StdErr { message }
            | LogOutput::Console { message } => String::from_utf8_lossy(&message).to_string(),
            _ => String::new(),
        };
        print!("MSG: {}", msg);
        if !msg.is_empty() {
            println!("DATA: {:?}", msg);
            task_log!("{}", msg);
        }
    }
    TASK_LOGGERS.remove(&task_id);
    Ok(())
}
