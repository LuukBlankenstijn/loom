use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("failed precondition: {0}")]
    FailedPrecondition(String),

    #[error("internal: {0}")]
    Internal(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),
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
            AppError::Http(e) => {
                tracing::error!(error = ?e, "http client error");
                Status::internal("internal http error")
            }
        }
    }
}
