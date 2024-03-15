//use std::{thread, time};

use std::fs::File;
use std::io::prelude::*;
use std::net::TcpListener;
use std::process::Command;
use tracing::info;
use uuid::Uuid;
pub async fn dockerise_container(id: uuid::Uuid) {
    // Can now create docker container
    // Need to create logging system text
    println!("Got uuid {:?}", id);
    let p = get_available_port();
    let port: u16;
    match p {
        Some(val) => port = val,
        None => {
            eprintln!("No ports available");
            return;
        }
    }
    if let Err(e) = create_dockerfile(&id) {
        //TODO Change this to gRPC logging ?
        eprintln!("Error creating docker file {}", e);
        return;
    }
    info!("Created docker file with UUID:{}", id);

    if let Err(e) = build_docker_image(&id) {
        //TODO Change this to gRPC logging ?
        eprintln!("Error building Docker image: {}", e);
        return;
    }
    info!("Built docker image with UUID:{} & port {}", id, port);
    if let Err(e) = run_docker_container(&id, &port) {
        //TODO Change this to gRPC logging ?
        eprintln!("Error running Docker image: {}", e);
        return;
    }
    info!("Successfully started docker container");
}

fn local_port_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_available_port() -> Option<u16> {
    let mut x = 5000; // 5000-6000
    while x < 6000 {
        if local_port_available(x) {
            return Some(x);
        }
        x += 1;
    }
    return None;
}
fn run_docker_container(uuid: &Uuid, port: &u16) -> std::io::Result<()> {
    let image_name = format!("kappa-go:{}", uuid);
    let output = Command::new("docker")
        .args(&[
            "run",
            "--rm",
            "-p",
            format!("{}:5182", port).as_str(),
            &image_name,
        ])
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
    }

    Ok(())
}
fn build_docker_image(uuid: &Uuid) -> std::io::Result<()> {
    let image_name = format!("kappa-go:{}", uuid);
    //docker build -t kappa:UUID -f ./zip/UUID/Dockerfile ./zip/UUID

    let output = Command::new("docker")
        .args(&[
            "build",
            "-t",
            &image_name,
            "-f",
            &format!("./zip/{}/Dockerfile", uuid),
            &format!("./zip/{}", uuid),
        ])
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
    }

    Ok(())
}
fn create_dockerfile(uuid: &Uuid) -> Result<(), anyhow::Error> {
    let filename = format!("./zip/{}/Dockerfile", uuid);
    let dockerfile = r#"FROM golang:1.22.1-alpine as golang
WORKDIR /app
copy . . 
RUN go mod init main.go && go mod tidy && go get .
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
