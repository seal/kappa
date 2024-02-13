use crate::errors::error::AppError;
use crate::models;
use crate::models::success::Success;
use crate::models::user::CreateUser;
use axum::extract::State;
use axum::Extension;
use axum::{http::StatusCode, response::Json};
use serde::Serialize;
use sqlx::postgres::PgPool;
use tracing::info;
use uuid::Uuid;
pub async fn get_user(
    current_user: Extension<models::user::User>,
) -> Result<Json<models::success::Success>, AppError> {
    Ok(Json(Success {
        message: "hello".to_string(),
    }))
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<NewUserResponse>, AppError> {
    info!("Creating user with username {}", payload.username);
    let api_key = Uuid::new_v4();

    sqlx::query!(
        r#"
            insert into "user"(api_key, username)
            values ($1, $2)
        "#,
        api_key.to_string(),
        payload.username
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        info!("Error inserting user: {}", e);
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Error inserting into user: {}", e),
        }
    })?;

    Ok(Json(NewUserResponse {
        message: "successfully created user".to_string(),
        api_key: api_key.to_string(),
    }))
}
#[derive(Debug, Serialize)]
pub struct NewUserResponse {
    message: String,
    api_key: String,
}
