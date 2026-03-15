pub mod admin;
pub mod broadcast;
pub mod map;
pub mod station;
pub mod wallpaper;

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use tonic::Status;

use crate::error::AppError;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde_json::json;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InvalidArgument(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::FailedPrecondition(msg) => (StatusCode::PRECONDITION_FAILED, msg),
            AppError::Internal(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred".to_string(),
                )
            }
            AppError::AlreadyExists(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<AppError> for Status {
    fn from(err: AppError) -> Self {
        match &err {
            AppError::NotFound(msg) => Status::not_found(msg.clone()),
            AppError::InvalidArgument(msg) => Status::invalid_argument(msg.clone()),
            AppError::AlreadyExists(msg) => Status::already_exists(msg.clone()),
            AppError::FailedPrecondition(msg) => Status::failed_precondition(msg.clone()),
            AppError::Internal(msg) => Status::internal(msg.clone()),
            AppError::Database(e) => {
                tracing::error!(error = %e, "database error");
                Status::internal("internal database error")
            }
        }
    }
}

impl From<AppError> for StatusCode {
    fn from(value: AppError) -> Self {
        match &value {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidArgument(_) => StatusCode::BAD_REQUEST,
            AppError::AlreadyExists(_) => StatusCode::NOT_MODIFIED,
            AppError::FailedPrecondition(_) => StatusCode::PRECONDITION_FAILED,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub fn to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
