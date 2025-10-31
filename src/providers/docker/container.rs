use crate::logs::TASK_LOGGERS;
use crate::models::DevBox;
use crate::task_log;
use bollard::Docker;
use bollard::container::LogOutput;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::LogsOptions;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::info;

// use std::io::{Read, Write, stdout};
// use std::time::Duration;
// #[cfg(not(windows))]
// use termion::raw::IntoRawMode;
// #[cfg(not(windows))]
// use termion::{async_stdin, terminal_size};
// use tokio::io::AsyncWriteExt;
// use tokio::task::spawn;
// use tokio::time::sleep;

pub async fn create(
    docker: Arc<Docker>,
    devcontainer: &DevBox,
) -> Result<String, Box<dyn std::error::Error>> {
    task_log!("Creating container...");
    let image_config = ContainerCreateBody {
        image: Some(devcontainer.name.clone()),
        tty: Some(true),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        open_stdin: Some(true),
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

// pub async fn exec(docker: Arc<Docker>, id: String) -> Result<(), Box<dyn std::error::Error>> {
//     #[cfg(not(windows))]
//     let tty_size = terminal_size()?;
//     let exec = docker
//         .create_exec(
//             &id,
//             bollard::models::ExecConfig {
//                 attach_stdout: Some(true),
//                 attach_stderr: Some(true),
//                 attach_stdin: Some(true),
//                 tty: Some(true),
//                 cmd: Some(vec![String::from("sh")]),
//                 ..Default::default()
//             },
//         )
//         .await?
//         .id;
//     #[cfg(not(windows))]
//     if let bollard::exec::StartExecResults::Attached {
//         mut output,
//         mut input,
//     } = docker.start_exec(&exec, None).await?
//     {
//         // pipe stdin into the docker exec stream input
//         spawn(async move {
//             #[allow(clippy::unbuffered_bytes)]
//             let mut stdin = async_stdin().bytes();
//             loop {
//                 if let Some(Ok(byte)) = stdin.next() {
//                     input.write_all(&[byte]).await.ok();
//                 } else {
//                     sleep(Duration::from_nanos(10)).await;
//                 }
//             }
//         });

//         docker
//             .resize_exec(
//                 &exec,
//                 bollard::query_parameters::ResizeExecOptionsBuilder::default()
//                     .h(tty_size.1 as i32)
//                     .w(tty_size.0 as i32)
//                     .build(),
//             )
//             .await?;

//         // set stdout in raw mode so we can do tty stuff
//         let stdout = stdout();
//         let mut stdout = stdout.lock().into_raw_mode()?;

//         // pipe docker exec output into stdout
//         while let Some(Ok(output)) = output.next().await {
//             stdout.write_all(output.into_bytes().as_ref())?;
//             stdout.flush()?;
//         }
//     }
//     Ok(())
// }
