use std::fs::File;
use std::io::Write;

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
    info!("Received file with bytes length of {:?}\n", file_b.len());
    let rec = sqlx::query!(
        r#"
            insert into "container"(language, port)
            values ($1, $2)
            RETURNING container_id
        "#,
        query.language,
        1234,
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
    Ok(Json(ReturnMessage {
        message: "successfully created container {}".to_string(),
        container_id: rec.container_id.to_string(),
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

    //println!("{:?}", containers);

    Ok(Json(containers))
}
