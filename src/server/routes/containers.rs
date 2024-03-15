use crate::docker::docker::dockerise_container;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::path::PathBuf;

use crate::errors::error::AppError;
use crate::models::container::{Container, NewContainer, QueryContainer, ReturnMessage};
use crate::models::user::User;
use axum::body::Bytes;
use axum::extract::{Multipart, Query};
use axum::{extract::State, Extension};
use axum::{http::StatusCode, Json};
use sqlx::postgres::PgPool;
use tracing::info;

pub async fn trigger_container(
    State(pool): State<PgPool>,
    query: Query<QueryContainer>,
) -> Result<Json<ReturnMessage>, AppError> {
    let container = sqlx::query!(
        r#"
    SELECT * FROM container
    WHERE container_id = $1
    "#,
        uuid::Uuid::parse_str(&query.container_id).map_err(|e| {
            AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Invalid query param {} ", e),
            }
        })?
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        info!("Error fetching container: {}", e);
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Error fetching container: {}", e),
        }
    })?;
    info!("Got container {:?}", container);
    return Ok(Json(ReturnMessage {
        message: "Successfully triggered container".to_string(),
        container_id: query.container_id.clone(),
    }));
}
pub async fn new_container(
    State(pool): State<PgPool>,
    query: Query<NewContainer>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<Json<ReturnMessage>, AppError> {
    match query.language.as_str() {
        "go" => (),
        "node" => (),
        _ => {
            return Err(AppError {
                status_code: StatusCode::BAD_REQUEST,
                message: "invalid language".to_string(),
            })
        }
    }
    let mut file_b = Bytes::default();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name();
        match name {
            Some("file") => {
                file_b = field.bytes().await.map_err(|e| AppError {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!("Error reading file: {}", e),
                })?
            }
            _ => {
                return Err(AppError {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!("Error with field name {:?}", name),
                })
            }
        }
    }
    info!("Received file with bytes length of {:?}\n", file_b.len());
    let rec = sqlx::query!(
        r#"
            insert into "container"(language, user_id)
            values ($1::TEXT, $2::UUID)
            RETURNING container_id
        "#,
        query.language,
        user.user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        info!("Error inserting container: {}", e);
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Error inserting into container : {}", e),
        }
    })?;
    let mut file = File::create(format!("./zip/{}.zip", rec.container_id)).map_err(|e| {
        info!("Error saving file: {}", e);
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Error saving file {}", e),
        }
    })?;
    file.write_all(&file_b).map_err(|e| AppError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("Error saving file {}", e),
    })?;
    let zip_file = fs::File::open(format!("./zip/{}.zip", rec.container_id)).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    if archive.len() > 1 {
        return Err(AppError {
            status_code: StatusCode::BAD_REQUEST,
            message: "zip has too many files".to_string(),
        });
    }
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| AppError {
            status_code: StatusCode::BAD_REQUEST,
            message: format!("{} Error archive by index", e),
        })?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if let Some(p) = outpath.parent() {
            if !p.exists() {
                fs::create_dir_all(p).map_err(|e| AppError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: format!("{} Error create dir all", e),
                })?;
            }
        }
        let mut zip_outpath = PathBuf::from(format!("zip/{}", rec.container_id));
        zip_outpath.push(outpath);
        info!(
            "File extracted to \"{}\" ({} bytes)",
            zip_outpath.display(),
            file.size()
        );
        fs::create_dir(format!("zip/{}", rec.container_id)).map_err(|e| AppError {
            status_code: StatusCode::BAD_REQUEST,
            message: format!("{} Error create dir container ID ", e),
        })?;
        let mut outfile = fs::File::create(&zip_outpath).map_err(|e| AppError {
            status_code: StatusCode::BAD_REQUEST,
            message: format!("{} Error creating outfile ", e),
        })?;
        io::copy(&mut file, &mut outfile).map_err(|e| AppError {
            status_code: StatusCode::BAD_REQUEST,
            message: format!("{} Error copying data to outfile", e),
        })?;
        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&zip_outpath, fs::Permissions::from_mode(mode)).map_err(
                    |e| AppError {
                        status_code: StatusCode::BAD_REQUEST,
                        message: format!("{} Error setting permissions for outfile", e),
                    },
                )?;
            }
        }
    }

    dockerise_container(rec.container_id).await;
    Ok(Json(ReturnMessage {
        message: "successfully created container {}".to_string(),
        container_id: rec.container_id.to_string(),
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
    .map_err(|e| {
        info!("Error fetching containers: {}", e);
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Error fetching containers: {}", e),
        }
    })?;

    Ok(Json(containers))
}
