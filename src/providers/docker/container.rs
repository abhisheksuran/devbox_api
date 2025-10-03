use bollard::Docker;
use bollard::models::ContainerCreateBody;
use models::DevContainer;

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
