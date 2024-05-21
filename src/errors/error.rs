use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;

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

//use std::zip::result::ZipError;
use thiserror::Error;
//     FileReadError(#[from] io::Error),
#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Invalid query param: {0}")]
    InvalidQueryParam(#[from] uuid::Error),
    #[error("Invalid language")]
    InvalidLanguage,
    #[error("Invalid field name: {0}")]
    InvalidFieldName(String),
    #[error("File read error: {0}")]
    FileReadError(String),
    #[error("Directory creation error: {0}")]
    DirectoryCreationError(String),
    #[error("File save error: {0}")]
    FileSaveError(String),
    #[error("Zip archive error: {0}")]
    ZipArchiveError(String),
    #[error("Dockerise container error: {0}")]
    DockeriseContainerError(String),
    #[error("Database insertion error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Database error: {0}")]
    FailedProxyRequest(#[from] reqwest::Error),
    #[error("Container not found or does not belong to user")]
    ContainerNotFound,
    #[error("Could not remove image / container: {0}")]
    DockerError(#[from] bollard::errors::Error),
}

/*
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
            CustomError::FailedBodyRead(msg) => write!(f, "Failed body read {}", msg),
            CustomError::FailedProxyRequest(msg) => write!(f, "failed proxy request {}", msg),
            CustomError::ContainerNotFound => {
                write!(f, "Container not found or does not belong to user")
            }
        }
    }
}

*/
use backtrace::Backtrace;

impl From<CustomError> for AppError {
    fn from(err: CustomError) -> Self {
        let status_code = match &err {
            CustomError::InvalidQueryParam(_) => StatusCode::BAD_REQUEST,
            CustomError::InvalidLanguage => StatusCode::BAD_REQUEST,
            CustomError::InvalidFieldName(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::FileReadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::DirectoryCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::FileSaveError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ZipArchiveError(_) => StatusCode::BAD_REQUEST,
            CustomError::DockeriseContainerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ContainerNotFound => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::FailedProxyRequest(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::DockerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let backtrace = Backtrace::new();
        tracing::error!(error.cause = ?err, error.backtrace = ?backtrace);
        AppError {
            status_code,
            message: err.to_string(),
        }
    }
}
