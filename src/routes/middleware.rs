use axum::http::StatusCode;
use sqlx::postgres::PgPool;

use crate::errors::error::AppError;
use crate::models::container::Container;
use crate::models::user::User;
use axum::extract::Request;
use axum::{extract::State, middleware::Next, response::Response};

use tracing::info;

pub async fn trigger_auth(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_key = request
        .headers()
        .get("api-key")
        .map(|value| value.to_str())
        .transpose()
        .map_err(|e| {
            info!("Error extracting API key: {}", e);
            AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Error extracting API key: {}", e),
            }
        })?
        .unwrap_or_else(|| "fix");
    let user: Result<User, sqlx::Error> = sqlx::query_as!(
        User,
        r#"
    SELECT * FROM "user"
    WHERE api_key = $1
    "#,
        api_key,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        info!("Error finding user: {}", e);
        e
    });
    if let Ok(user) = user {
        let container_id = request
            .headers()
            .get("container")
            .map(|value| value.to_str())
            .transpose()
            .map_err(|e| {
                info!("Error extracting container ID: {}", e);
                AppError {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!("Error extracting container ID: {}", e),
                }
            })?;
        if let Some(container_id) = container_id {
            let container: Result<Container, sqlx::Error> = sqlx::query_as!(
                Container,
                r#"
            SELECT * FROM container
            WHERE container_id = $1 AND user_id = $2
            "#,
                uuid::Uuid::parse_str(container_id).map_err(|e| {
                    AppError {
                        status_code: StatusCode::BAD_REQUEST,
                        message: format!("Invalid container ID: {}", e),
                    }
                })?,
                user.user_id,
            )
            .fetch_one(&pool)
            .await;
            if let Ok(container) = container {
                request.extensions_mut().insert(user);
                request.extensions_mut().insert(container);
            } else {
                return Err(AppError {
                    status_code: StatusCode::UNAUTHORIZED,
                    message: "Container does not belong to the user".to_string(),
                });
            }
        } else {
            return Err(AppError {
                status_code: StatusCode::BAD_REQUEST,
                message: "Missing container ID".to_string(),
            });
        }
    } else {
        return Err(AppError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "No user found".to_string(),
        });
    }
    Ok(next.run(request).await)
}

pub async fn api_key_auth(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_key = request
        .headers()
        .get("api-key")
        .map(|value| value.to_str())
        .transpose()
        .map_err(|e| {
            info!("Error extracting API key: {}", e);
            AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Error extracting API key: {}", e),
            }
        })?
        .unwrap_or_else(|| "fix");
    info!("api key {}", api_key);
    let user: Result<User, sqlx::Error> = sqlx::query_as!(
        User,
        r#"
    SELECT * FROM "user"
    WHERE api_key = $1
    "#,
        api_key,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        info!("Error finding user: {}", e);
        e
    });
    if let Ok(user) = user {
        request.extensions_mut().insert(user);
    } else {
        return Err(AppError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "No user found".to_string(),
        });
    }
    Ok(next.run(request).await)
}
