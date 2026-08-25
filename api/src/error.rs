use axum::{http::StatusCode, response::Response};
pub enum AppError {
    Db(sqlx::Error),
    NotFound(String),
    Conflict(String),
}
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        if matches!(error, sqlx::Error::RowNotFound) {
            return AppError::NotFound("not found".to_string());
        }
        if let sqlx::Error::Database(ref err) = error {
            if err.is_unique_violation() {
                return AppError::Conflict("Already exists or referenced".to_string());
            }
        }
        AppError::Db(error)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Db(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message).into_response(),
        }
    }
}
