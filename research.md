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

