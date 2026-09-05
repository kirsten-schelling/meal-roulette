use axum::{http::StatusCode, response::Response};
pub enum AppError {
    Db(sqlx::Error),
    NotFound(String),
    Conflict(String),
    #[expect(dead_code)]
    BadRequest(String),
}

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
        match self {
            AppError::Db(error) => {
                eprintln!("[api] DB error: {error:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            AppError::NotFound(message) => {
                eprintln!("[api] 404: {message}");
                (StatusCode::NOT_FOUND, message).into_response()
            }
            AppError::Conflict(message) => {
                eprintln!("[api] 409: {message}");
                (StatusCode::CONFLICT, message).into_response()
            }
            AppError::BadRequest(message) => {
                eprintln!("[api] 400: {message}");
                (StatusCode::BAD_REQUEST, message).into_response()
            }
        }
    }
}
