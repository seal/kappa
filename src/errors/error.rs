use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;
use std::fmt;
use tracing::error;

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status_code, Json(json!({"error": self.message}))).into_response()
    }
}

#[derive(Debug)]
pub enum CustomError {
    InvalidQueryParam(String),
    InvalidLanguage,
    InvalidFieldName(String),
    FileReadError(String),
    DirectoryCreationError(String),
    FileSaveError(String),
    ZipArchiveError(String),
    DockeriseContainerError(String),
    DatabaseInsertionError(String),
    DatabaseFetchError(String),
    DatabaseDeletionError(String),
    ContainerNotFound,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomError::InvalidQueryParam(msg) => write!(f, "Invalid query param: {}", msg),
            CustomError::InvalidLanguage => write!(f, "Invalid language"),
            CustomError::InvalidFieldName(name) => write!(f, "Invalid field name: {}", name),
            CustomError::FileReadError(msg) => write!(f, "File read error: {}", msg),
            CustomError::DirectoryCreationError(msg) => {
                write!(f, "Directory creation error: {}", msg)
            }
            CustomError::FileSaveError(msg) => write!(f, "File save error: {}", msg),
            CustomError::ZipArchiveError(msg) => write!(f, "Zip archive error: {}", msg),
            CustomError::DockeriseContainerError(msg) => {
                write!(f, "Dockerise container error: {}", msg)
            }
            CustomError::DatabaseInsertionError(msg) => {
                write!(f, "Database insertion error: {}", msg)
            }
            CustomError::DatabaseFetchError(msg) => write!(f, "Database fetch error: {}", msg),
            CustomError::DatabaseDeletionError(msg) => {
                write!(f, "Database deletion error: {}", msg)
            }
            CustomError::ContainerNotFound => {
                write!(f, "Container not found or does not belong to user")
            }
        }
    }
}

impl From<CustomError> for AppError {
    fn from(err: CustomError) -> Self {
        let status_code = match err {
            CustomError::InvalidQueryParam(_) => StatusCode::BAD_REQUEST,
            CustomError::InvalidLanguage => StatusCode::BAD_REQUEST,
            CustomError::InvalidFieldName(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::FileReadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::DirectoryCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::FileSaveError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ZipArchiveError(_) => StatusCode::BAD_REQUEST,
            CustomError::DockeriseContainerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::DatabaseInsertionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::DatabaseFetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::DatabaseDeletionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ContainerNotFound => StatusCode::INTERNAL_SERVER_ERROR,
        };
        error!("{}", err);
        AppError {
            status_code,
            message: err.to_string(),
        }
    }
}
