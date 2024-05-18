https://tomasz.janczuk.org/2018/03/how-to-build-your-own-serverless-platform.html
Anyhow errors -> https://github.com/tokio-rs/axum/tree/main/examples/anyhow-error-response
MySQL -> https://github.com/wpcodevo/rust-axum-mysql
Moved to postgres due to uuid funcs etc
Running postgres locally through docker
 curl localhost:3000/containers

curl -H 'Content-Type: application/json' -d '{"language":"go", "port":1234}' -X POST localhost:3000/containers
[Container { container_id: 9d58e709-c84e-46ae-b1b4-df6cc8959e64, language: "go", port: 1234 }]

Errors:
 .map_err(|e| {
        info!("Error inserting container: {}", e);
        AppError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("Error inserting into container : {}", e),
    }})?;

// Small golang docker image
https://medium.com/@pavelfokin/how-to-build-a-minimal-golang-docker-image-b4a1e51b03c8
➜  minimal-docker git:(main) ✗ docker build -t helloworld-small -f Dockerfile.small .
[+] Building 8.0s (14/14) FINISHED
...
...
...➜  minimal-docker git:(main) ✗ docker image ls helloworld-small
REPOSITORY         TAG       IMAGE ID       CREATED         SIZE
helloworld-small   latest    3b035dc2c7e4   5 seconds ago   8.6MB➜  minimal-docker git:(main) ✗ docker run -ti -p 8000:8000 helloworld-small:latest
Server listen 8000...
Tried to time it, had to stop due to terrible wifi 
3:41 minutes
After initial download builds took 0.01
Changing port took 11s to re-build

   docker-test   git:(main) ✗ docker image ls
REPOSITORY                          TAG             IMAGE ID       CREATED         SIZE
kappa-small                         latest          cb285a3d696e   2 minutes ago   10.3MB
golang                              1.22.1-alpine   8843ca6fa27e   4 days ago      230MB
dpage/pgadmin4                      latest          3158f8135bcd   5 days ago      472MB
postgres                            latest          eb634efa7ee4   2 weeks ago     431MB
hello-world                         latest          d2c94e258dcb   10 months ago   13.3kB
golang                              1.18-alpine     a77f45e5f987   14 months ago   330MB
gcr.io/distroless/static-debian11   latest          8c4272443506   N/A             2.56MB

after docker run -p 5182:5182 kappa-small 

curl -X "POST" 127.0.0.1:5183 

- returns "here" ( as expected ) 

Old docker code:
```
fn run_docker_container(uuid: &Uuid, port: &u16) -> Result<()> {
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
        return Err(anyhow!("Error running docker command: {}", err));
    }

    Ok(())
}
fn build_docker_image(uuid: &Uuid) -> Result<()> {
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
        return Err(anyhow!("error building docker image {}", err));
    }

    Ok(())
}
fn create_dockerfile(uuid: &Uuid) -> Result<(), anyhow::Error> {
    let filename = format!("./zip/{}/Dockerfile", uuid);
    let dockerfile = r#"FROM golang:1.22.1-alpine as golang
WORKDIR /app
COPY . . 
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
```
Got em running with docker 
not yet triggering via rust 

trigger manually via:
```
curl -X POST http://localhost:5182
curl -X POST http://localhost:5182?id=123 -H "Custom-Header: value"
curl -X POST http://localhost:5182 -H "Content-Type: application/json" -d '{"message": "Hello, World!"}'
```
Change ports to docker ports
Also context returns a 404, I believe due to handle being a get request




Trigger container ( no stream) 
```
pub async fn trigger_container(
    Extension(container): Extension<Container>,
    method: Method,
    query: RawQuery,
    //uri: Uri, Not needed
    header_map: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, HeaderMap, String), AppError> {
    let target_uri = format!("127.0.0.1:{}", container.port.unwrap_or(0));
    let query = query.0.map(|query_str| {
        serde_qs::from_str::<HashMap<String, String>>(&query_str)
            .unwrap_or_else(|err| [("error".to_string(), err.to_string())].into())
    });
    let body_string = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(_) => "".to_string(), // No body is fine
    };
    let client = Client::new();
    let response = client
        .request(method, &target_uri)
        .query(&query)
        .headers(header_map)
        .body(body_string)
        .send()
        .await
        .map_err(|e| CustomError::FailedProxyRequest(e.to_string()))?;

    let status_code = response.status();
    let headers = response.headers().clone();
    let response_body = response
        .text()
        .await
        .map_err(|e| CustomError::FailedBodyRead(e.to_string()))?;
    Ok((status_code, headers, response_body))
}
```
REMOVE CACHE THING
