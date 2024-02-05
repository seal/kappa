use crate::errors::error::AppError;
use crate::models::user::{CreateUser, User};
use axum::extract::{MatchedPath, Request};
use axum::{http::StatusCode, response::Json};
use tracing::info;
pub async fn error_handler(req: Request) -> Result<(), AppError> {
    try_thing(req).await?;
    Ok(())
}

async fn try_thing(req: Request) -> Result<(), AppError> {
    info_req(req);
    Err(AppError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "it failed!".to_string(),
    })
}
fn info_req(req: Request) {
    info!(
        "Error occurred processing request: method={}, uri={}, matched_path={}",
        req.method().clone(),
        req.uri().to_owned(),
        req.extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str().to_owned())
            .unwrap_or_default()
    );
}
pub async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1337,
        username: payload.username.clone(),
    };

    (StatusCode::CREATED, Json(user))
}
