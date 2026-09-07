use axum::{http::StatusCode, response::Response};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Db(sqlx::Error),
    NotFound(String),
    Conflict(String),
    BadRequest(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Db(error) => write!(f, "{error}"),
            AppError::NotFound(message)
            | AppError::Conflict(message)
            | AppError::BadRequest(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for AppError {}

pub fn is_foreign_key_violation(error: &sqlx::Error) -> Option<&dyn sqlx::error::DatabaseError> {
    error
        .as_database_error()
        .filter(|db| db.is_foreign_key_violation())
}

pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|db| db.is_unique_violation())
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        if matches!(error, sqlx::Error::RowNotFound) {
            return AppError::NotFound("not found".to_string());
        }
        if is_unique_violation(&error) {
            return AppError::Conflict("Already exists".to_string());
        }
        if let Some(db) = is_foreign_key_violation(&error) {
            let constraint = db.constraint().unwrap_or_default();
            return AppError::Conflict(format!("Still in use by another record: {constraint}"));
        }
        AppError::Db(error)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        eprintln!("[api] {status}: {self}");
        (status, self.to_string()).into_response()
    }
}
