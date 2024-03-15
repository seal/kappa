use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status_code, Json(json!({"error": self.message}))).into_response()
    }
}
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
}

/*
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let anyhow_err: anyhow::Error = err.into();
        info!(%anyhow_err);
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Something went wrong: {}", anyhow_err),
        }
    }
}
*/
