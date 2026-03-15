mod admin;
mod map;

pub use admin::*;
pub use map::MapHandler;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use tonic::Status;

use crate::error::AppError;

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

pub fn to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
