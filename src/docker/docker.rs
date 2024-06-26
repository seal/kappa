use anyhow::{anyhow, Error, Result};
use async_std::stream::StreamExt;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, PruneContainersOptions,
    RemoveContainerOptions, StartContainerOptions,
};
use bollard::image::{BuildImageOptions, PruneImagesOptions, RemoveImageOptions};
use bollard::models::HostConfig;
use bollard::models::PortBinding;
use bollard::Docker;
use bytes::Bytes;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpListener;
use tar::Builder;
use tracing::{error, info};
use uuid::Uuid;

use crate::errors::error::CustomError;
pub async fn delete_docker_container_and_image(
    container_id: &Uuid,
    container_port: &i32,
) -> Result<(), CustomError> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| CustomError::DockerError(e))?;
    let container_name = format!("kappa-container-{}", container_id);
    let image_name = format!("kappa-go:{}", container_id);

    // Force remove the Docker container
    match docker
        .remove_container(
            &container_name,
            Some(RemoveContainerOptions {
                v: true,
                force: true,
                ..Default::default()
            }),
        )
        .await
    {
        Ok(_) => (),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404,
            message: _,
        }) => {
            error!("Ignoring container docker remove error ");
        }
        Err(e) => return Err(CustomError::DockerError(e)),
    }
    // Remove the Docker image
    match docker
        .remove_image(
            &image_name,
            Some(RemoveImageOptions {
                force: true,
                noprune: false,
            }),
            None,
        )
        .await
    {
        Ok(_) => (),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404,
            message: _,
        }) => {
            error!("Ignoring image docker remove error ");
        }
        Err(e) => return Err(CustomError::DockerError(e)),
    }
    // Prune containers
    docker
        .prune_containers(None::<PruneContainersOptions<String>>)
        .await
        .map_err(|e| CustomError::DockerError(e))?;
    // Prune images
    docker
        .prune_images(None::<PruneImagesOptions<String>>)
        .await
        .map_err(|e| CustomError::DockerError(e))?;
    // Remove the container files
    let _ = std::fs::remove_dir_all(format!("./zip/{}", container_id));
    let _ = std::fs::remove_file(format!("./zip/{}.zip", container_id));
    remove_port_from_used(container_port.clone() as u16);

    Ok(())
}
pub async fn dockerise_container(id: uuid::Uuid) -> Result<i32, Error> {
    let p = get_available_port();
    let port: u16;
    match p {
        Some(val) => port = val,
        None => {
            return Err(anyhow!("No ports available"));
        }
    }
    if let Err(e) = create_dockerfile(&id) {
        return Err(anyhow!("Error creating docker file {}", e));
    }
    info!("Created docker file with UUID:{}", id);

    if let Err(e) = run_docker_container(&id, &port).await {
        return Err(anyhow!("Error running Docker image: {}", e));
    }
    info!("Successfully built and started docker container");
    Ok((port).try_into().unwrap())
}
use std::sync::Mutex;

lazy_static! {
    static ref USED_PORTS: Mutex<Vec<u16>> = Mutex::new(Vec::new());
}

fn local_port_available(port: u16) -> bool {
    let used_ports = USED_PORTS.lock().unwrap();
    !used_ports.contains(&port)
        && match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => true,
            Err(_) => false,
        }
}

fn get_available_port() -> Option<u16> {
    let mut x = 5000; // 5000-6000
    while x < 6000 {
        if local_port_available(x) {
            add_port_to_used(x);
            return Some(x);
        }
        x += 1;
    }
    None
}

fn add_port_to_used(port: u16) {
    let mut used_ports = USED_PORTS.lock().unwrap();
    used_ports.push(port);
}

fn remove_port_from_used(port: u16) {
    let mut used_ports = USED_PORTS.lock().unwrap();
    if let Some(index) = used_ports.iter().position(|&p| p == port) {
        used_ports.remove(index);
    }
}

async fn run_docker_container(uuid: &Uuid, port: &u16) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    let image_name = format!("kappa-go:{}", uuid);
    // Check if a container with the same name already exists
    let container_name = format!("kappa-container-{}", uuid);
    let existing_container = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: HashMap::from([("name".to_string(), vec![container_name.clone()])]),
            ..Default::default()
        }))
        .await?;

    if !existing_container.is_empty() {
        // Start the existing container
        docker
            .start_container(
                &existing_container[0]
                    .id
                    .as_ref()
                    .unwrap_or(&"No container id ??? ".to_string()),
                None::<StartContainerOptions<String>>,
            )
            .await?;
        return Ok(());
    }
    //Bollard requires a tar file to have local Dockerfiles
    let tar_bytes = {
        let mut tar_buffer = Vec::new();
        {
            let mut tar_builder = Builder::new(&mut tar_buffer);

            // Add the files from ./zip/{uuid}/* to the tar archive
            let zip_path = format!("./zip/{}", uuid);
            tar_builder.append_dir_all(".", zip_path)?;
            tar_builder.finish()?;
        }
        Bytes::from(tar_buffer)
    };
    // Build the Docker image
    let build_options = BuildImageOptions {
        dockerfile: "Dockerfile".to_string(),
        t: image_name.clone(),
        q: true,
        rm: true,
        ..Default::default()
    };

    let mut image_build_stream = docker.build_image(build_options, None, Some(tar_bytes));
    while let Some(build_result) = image_build_stream.next().await {
        match build_result {
            Ok(output) => {
                info!(
                    "Building container {} progress : {}",
                    uuid,
                    output.progress.unwrap_or("No progress".to_string())
                );
            }
            Err(e) => return Err(anyhow!("Error building Docker image: {}", e)),
        }
    }

    let container_config = Config {
        image: Some(image_name),
        //exposed_ports: Some(exposed_ports),
        host_config: Some(HostConfig {
            port_bindings: Some(HashMap::from([(
                "5182/tcp".to_string(),
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(format!("{}", port)),
                }]),
            )])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let container_options = Some(CreateContainerOptions {
        name: format!("kappa-container-{}", uuid),
        ..Default::default()
    });
    let container_create_response = docker
        .create_container(container_options, container_config)
        .await?;
    let container_id = container_create_response.id;
    docker
        .start_container(&container_id, None::<StartContainerOptions<String>>)
        .await?;
    Ok(())
}
// TODO REMOVE CACHE
fn create_dockerfile(uuid: &Uuid) -> Result<(), anyhow::Error> {
    let filename = format!("./zip/{}/Dockerfile", uuid);
    let dockerfile = r#"FROM golang:1.22.1-alpine as golang
WORKDIR /app
COPY . . 
RUN go mod init main.go && go mod tidy 
RUN go get .
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /server . 
FROM gcr.io/distroless/static-debian11
COPY --from=golang /server .
EXPOSE 5182
CMD ["/server"]
"#;
    let mut file = File::create(filename)?;
    file.write_all(dockerfile.as_bytes())?;
    Ok(())
}
