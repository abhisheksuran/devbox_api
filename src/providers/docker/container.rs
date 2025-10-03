use bollard::Docker;
use bollard::models::ContainerCreateBody;
use models::DevContainer;

use std::io::{Read, Write, stdout};
use std::time::Duration;
#[cfg(not(windows))]
use termion::raw::IntoRawMode;
#[cfg(not(windows))]
use termion::{async_stdin, terminal_size};
use tokio::io::AsyncWriteExt;
use tokio::task::spawn;
use tokio::time::sleep;

pub async fn create(
    docker: Docker,
    devcontainer: &DevContainer,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Creating container...");
    let alpine_config = ContainerCreateBody {
        image: Some(String::from("devbox")),
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
            alpine_config,
        )
        .await?
        .id;

    Ok(id)
}

pub async fn start(docker: Docker, id: String) -> Result<(), Box<dyn std::error::Error>> {
    docker
        .start_container(
            &id,
            None::<bollard::query_parameters::StartContainerOptions>,
        )
        .await?;
    Ok(())
}

pub async fn remove(docker: Docker, id: String) -> Result<(), Box<dyn std::error::Error>> {
    docker
        .remove_container(
            &id,
            Some(
                bollard::query_parameters::RemoveContainerOptionsBuilder::default()
                    .force(true)
                    .build(),
            ),
        )
        .await?;
    Ok(())
}

pub async fn exec(docker: Docker, id: String) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    let tty_size = terminal_size()?;
    let exec = docker
        .create_exec(
            &id,
            bollard::models::ExecConfig {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                attach_stdin: Some(true),
                tty: Some(true),
                cmd: Some(vec![String::from("sh")]),
                ..Default::default()
            },
        )
        .await?
        .id;
    #[cfg(not(windows))]
    if let bollard::exec::StartExecResults::Attached {
        mut output,
        mut input,
    } = docker.start_exec(&exec, None).await?
    {
        // pipe stdin into the docker exec stream input
        spawn(async move {
            #[allow(clippy::unbuffered_bytes)]
            let mut stdin = async_stdin().bytes();
            loop {
                if let Some(Ok(byte)) = stdin.next() {
                    input.write_all(&[byte]).await.ok();
                } else {
                    sleep(Duration::from_nanos(10)).await;
                }
            }
        });

        docker
            .resize_exec(
                &exec,
                bollard::query_parameters::ResizeExecOptionsBuilder::default()
                    .h(tty_size.1 as i32)
                    .w(tty_size.0 as i32)
                    .build(),
            )
            .await?;

        // set stdout in raw mode so we can do tty stuff
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode()?;

        // pipe docker exec output into stdout
        while let Some(Ok(output)) = output.next().await {
            stdout.write_all(output.into_bytes().as_ref())?;
            stdout.flush()?;
        }
    }
    Ok(())
}
