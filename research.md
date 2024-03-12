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


