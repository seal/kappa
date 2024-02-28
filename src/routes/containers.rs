use std::fs::File;
use std::io::Write;

use crate::errors::error::AppError;
use crate::models::container::{Container, NewContainer};
use crate::models::success::Success;
use crate::models::user::User;
use axum::body::Bytes;
use axum::extract::{Multipart, Query};
use axum::{extract::State, Extension};
use axum::{http::StatusCode, Json};
//use futures_util::stream::StreamExt;
use sqlx::postgres::PgPool;
use tracing::info;

pub async fn new_container(
    State(pool): State<PgPool>,
    query: Query<NewContainer>,
    mut multipart: Multipart,
) -> Result<Json<Success>, AppError> {
    let mut file_b = Bytes::default();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "file" => {
                file_b = field.bytes().await.map_err(|e| AppError {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!("Error reading file: {}", e),
                })?
            }
            _ => {
                return Err(AppError {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!("Error with field name {}", name),
                })
            }
        }
    }
    info!("Received file with bytes {:?}", file_b);
    //File::create(path)
    let rec = sqlx::query!(
        r#"
            insert into "container"(language, port)
            values ($1, $2)
            RETURNING container_id
        "#,
        query.language,
        query.port
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
    let mut file = File::create(format!("./files/{}", rec.container_id)).map_err(|e| AppError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("Error saving file {}", e),
    })?;
    file.write_all(&file_b).map_err(|e| AppError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("Error saving file {}", e),
    })?;
    Ok(Json(Success {
        message: format!(
            "successfully created container with id {}",
            rec.container_id
        ),
    }))
}

//pub async fn get_containers(State(pool): State<PgPool>) -> Result<Json<Vec<Container>>, AppError> {
pub async fn get_containers(
    Extension(user): Extension<User>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Container>>, AppError> {
    println!("User in func{:?}", user);
    let containers = sqlx::query_as!(
        Container,
        r#"
        SELECT * FROM container
        "#
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

    println!("{:?}", containers);

    Ok(Json(containers))
}
