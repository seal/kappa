use crate::errors::error::AppError;
use crate::models::container::{Container, NewContainer};
use crate::models::success::Success;
use axum::extract::State;
use axum::{http::StatusCode, Json};
use sqlx::postgres::PgPool;
use tracing::info;

pub async fn new_container(
    State(pool): State<PgPool>,
    Json(payload): Json<NewContainer>,
    // req: Request,
) -> Result<Json<Success>, AppError> {
    sqlx::query!(
        r#"
            insert into "container"(language, port)
            values ($1, $2)
        "#,
        payload.language,
        payload.port
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        info!("Error inserting container: {}", e);
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Error inserting into container : {}", e),
        }
    })?;

    Ok(Json(Success {
        message: "successfully created container".to_string(),
    }))
}
pub async fn get_containers(State(pool): State<PgPool>) -> Result<Json<Vec<Container>>, AppError> {
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
