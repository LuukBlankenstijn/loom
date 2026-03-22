pub mod admin;
pub mod broadcast;
pub mod http;
pub mod map;
pub mod station;

use axum::http::StatusCode;
use tonic::{Request, Status};

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

pub fn combined_auth_interceptor(
    secret_option: Option<String>,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
    let expected_header = secret_option.map(|s| format!("Bearer {}", s));
    move |req: Request<()>| {
        let expected = match &expected_header {
            Some(e) => e,
            None => return Ok(req),
        };

        let metadata = req.metadata();

        if metadata.contains_key("x-proxy-authorized") {
            return Ok(req);
        }

        let auth_header = metadata.get("authorization").and_then(|h| h.to_str().ok());

        if auth_header == Some(expected) {
            Ok(req)
        } else {
            tracing::warn!("Unauthorized gRPC access attempt");
            Err(Status::unauthenticated("Missing or invalid credentials"))
        }
    }
}
