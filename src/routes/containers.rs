use crate::docker::docker::{delete_docker_container_and_image, dockerise_container};
use axum::http::{HeaderMap, Method};
use axum::response::Result;
use reqwest::Client;
use tracing::error;

use axum::{
    body::Bytes,
    extract::{Extension, Query},
    Json,
};
use http::StatusCode;
use std::collections::HashMap;
use std::fs::{self, create_dir_all, File};
use std::io::Write;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

use crate::errors::error::{AppError, CustomError};
use crate::models::container::{Container, NewContainer, QueryContainer, ReturnMessage};
use crate::models::user::User;
use axum::extract::State;
use axum::extract::{Multipart, RawQuery};
use sqlx::postgres::PgPool;
use tracing::info;
use uuid::Uuid;
pub async fn delete_container(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    query: Query<QueryContainer>,
) -> Result<Json<ReturnMessage>, AppError> {
    let container_id = uuid::Uuid::parse_str(&query.container_id)
        .map_err(|e| CustomError::InvalidQueryParam(e))?;

    // Check if the container exists and belongs to the user
    let container = sqlx::query!(
        r#"
        SELECT * FROM container
        WHERE container_id = $1 AND user_id = $2
        "#,
        container_id,
        user.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| CustomError::DatabaseError(e))?;

    if container.is_none() {
        return Err(CustomError::ContainerNotFound.into());
    }

    // Delete the container from the database
    sqlx::query!(
        r#"
        DELETE FROM container
        WHERE container_id = $1
        "#,
        container_id
    )
    .execute(&pool)
    .await
    .map_err(|e| CustomError::DatabaseError(e))?;
    delete_docker_container_and_image(&container_id).await?;
    // Remove the container files and docker container
    // (You may need to implement this part based on your specific requirements)

    Ok(Json(ReturnMessage {
        message: "Successfully deleted container".to_string(),
        container_id: container_id.to_string(),
    }))
}
#[tracing::instrument]
pub async fn trigger_container(
    State(pool): State<PgPool>,
    Extension(container): Extension<Container>,
    method: Method,
    query: RawQuery,
    header_map: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, HeaderMap, String), AppError> {
    let target_uri = format!("http://127.0.0.1:{}", container.port.unwrap_or(0));
    let query_map: HashMap<String, String> = query
        .0
        .as_deref()
        .map(|query_str| {
            serde_qs::from_str(query_str)
                .unwrap_or_else(|err| [("error".to_string(), err.to_string())].into())
        })
        .unwrap_or_default();

    let client = Client::new();
    info!("Sending request to {}", target_uri);
    let response = client
        .request(method.clone(), &target_uri)
        .query(&query_map)
        .headers(header_map.clone())
        .body(body.clone())
        .send()
        .await;

    match response {
        Ok(response) => {
            info!(
                "Got response from container with id {}",
                container.container_id
            );
            let status_code = response.status();
            let headers = response.headers().clone();
            let response_body = response
                .text()
                .await
                .map_err(CustomError::FailedProxyRequest)?;
            Ok((status_code, headers.clone(), response_body))
        }
        Err(e) if e.is_connect() => {
            info!(
                "First connection failed, retrying after starting container, ID {}",
                container.container_id
            );
            let port = dockerise_container(Uuid::from(container.container_id))
                .await
                .map_err(|e| CustomError::DockeriseContainerError(e.to_string()))?;
            let response = client
                .request(method, format!("http://127.0.0.1:{}", port))
                .query(&query_map)
                .headers(header_map)
                .body(body)
                .send()
                .await
                .map_err(|e| {
                    error!(
                        "failed second proxy request for ID {}",
                        container.container_id
                    );
                    CustomError::FailedProxyRequest(e)
                })?;
            let status_code = response.status();
            let headers = response.headers().clone();
            let response_body = response
                .text()
                .await
                .map_err(CustomError::FailedProxyRequest)?;
            sqlx::query!(
                r#"
    UPDATE "container"
    SET port = $1::INTEGER
    WHERE container_id = $2::UUID
    "#,
                port,
                container.container_id
            )
            .execute(&pool)
            .await
            .map_err(CustomError::DatabaseError)?;
            Ok((status_code, headers.clone(), response_body))
        }
        Err(e) => Err(CustomError::FailedProxyRequest(e).into()),
    }
}
/*
 *
pub async fn trigger_container(
    State(pool): State<PgPool>,
    query: Query<QueryContainer>,
) -> Result<Json<ReturnMessage>, AppError> {
    let container = sqlx::query!(
        r#"
    SELECT * FROM container
    WHERE container_id = $1
    "#,
        uuid::Uuid::parse_str(&query.container_id)
            .map_err(|e| { CustomError::InvalidQueryParam(e.to_string()) })?
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| CustomError::DatabaseFetchError(e.to_string()))?;
    info!("Got container {:?}", container);
    return Ok(Json(ReturnMessage {
        message: "Successfully triggered container".to_string(),
        container_id: query.container_id.clone(),
    }));
}
*/
pub async fn new_container(
    State(pool): State<PgPool>,
    query: Query<NewContainer>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<Json<ReturnMessage>, AppError> {
    match query.language.as_str() {
        "go" => (),
        "node" => (),
        _ => return Err(CustomError::InvalidLanguage.into()),
    }
    let container_id = Uuid::new_v4();
    let mut file_b = Bytes::default();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name();
        match name {
            Some("file") => {
                file_b = field
                    .bytes()
                    .await
                    .map_err(|e| CustomError::FileReadError(e.to_string()))?
            }
            _ => {
                return Err(CustomError::InvalidFieldName("Field name not file".to_string()).into())
            }
        }
    }
    info!("Received file with bytes length of {:?}\n", file_b.len());
    // on fresh clone, the zip file doesn't exist as it's in .gitignore ( don't want all the files
    // in the git repo)
    /*let mut file = File::create(format!("./zip/{}.zip", rec.container_id)).map_err(|e| {
        info!("Error saving file: {}", e);
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Error creating file {}", e),
        }
    })?;*/
    let mut file = match File::create(format!("./zip/{}.zip", container_id)) {
        Ok(file) => file,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                // Directory doesn't exist, create it
                if let Err(create_dir_err) = create_dir_all("./zip") {
                    return Err(
                        CustomError::DirectoryCreationError(create_dir_err.to_string()).into(),
                    );
                }

                // Try creating the file again
                match File::create(format!("./zip/{}.zip", container_id)) {
                    Ok(file) => file,
                    Err(e) => {
                        return Err(CustomError::FileSaveError(e.to_string()).into());
                    }
                }
            } else {
                // Other types of errors
                return Err(CustomError::FileSaveError(e.to_string()).into());
            }
        }
    };

    file.write_all(&file_b)
        .map_err(|e| CustomError::FileSaveError(e.to_string()))?;
    let zip_file = fs::File::open(format!("./zip/{}.zip", container_id)).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    //TODO remove ?
    //if archive.len() > 1 {
    //return Err(CustomError::ZipArchiveError("Zip has too many files ? ".to_string()).into());
    //}

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| CustomError::ZipArchiveError(format!("Error archive by index {e}")))?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let mut zip_outpath = PathBuf::from(format!("zip/{}", container_id));

        // Check if the outpath has a parent directory
        if let Some(parent) = outpath.parent() {
            // If the outpath has a parent directory, strip the top-level directory
            let stripped_path = match outpath.strip_prefix(parent) {
                Ok(path) => path.to_owned(),
                Err(_) => outpath.clone(),
            };
            zip_outpath.push(stripped_path);
        } else {
            // If the outpath doesn't have a parent directory, use the file name directly
            zip_outpath.push(outpath.file_name().unwrap());
        }

        if let Some(parent) = zip_outpath.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CustomError::DirectoryCreationError(format!("{e} create dir {parent:?}"))
            })?;
        }

        info!(
            "File extracted to \"{}\" ({} bytes)",
            zip_outpath.display(),
            file.size()
        );

        let mut outfile = fs::File::create(&zip_outpath).map_err(|e| {
            CustomError::DirectoryCreationError(format!("error creating outfile {e}"))
        })?;
        io::copy(&mut file, &mut outfile)
            .map_err(|e| CustomError::ZipArchiveError(format!("Copying data to outfile {e}")))?;
        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&zip_outpath, fs::Permissions::from_mode(mode)).map_err(
                    |e| CustomError::ZipArchiveError(format!("Permissions out file {e}")),
                )?;
            }
        }
    }
    let port = dockerise_container(container_id)
        .await
        .map_err(|e| CustomError::DockeriseContainerError(e.to_string()))?;

    // Previously returned container_id and it was generated by the DB, but this lead to orphaned
    // DB values if any-other part of the request failed.
    sqlx::query!(
        r#"
        insert into "container"(container_id, language, user_id, port, name)
        values ($1::UUID, $2::TEXT, $3::UUID, $4::INTEGER, $5::TEXT)
    "#,
        container_id,
        query.language,
        user.user_id,
        port,
        query.name
    )
    .execute(&pool)
    .await
    .map_err(|e| CustomError::DatabaseError(e))?;
    Ok(Json(ReturnMessage {
        message: "successfully created container".to_string(),
        container_id: container_id.to_string(),
    }))
}

pub async fn get_containers(
    Extension(user): Extension<User>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Container>>, AppError> {
    println!("User in func{:?}", user);
    let containers = sqlx::query_as!(
        Container,
        r#"
        SELECT * FROM container WHERE user_id = $1
        "#,
        user.user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| CustomError::DatabaseError(e))?;

    Ok(Json(containers))
}
