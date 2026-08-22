use axum::{http::StatusCode, response::Response};
pub enum AppError {
    Db(sqlx::Error),
}
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::Db(error)
    }
}
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Db(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
        }
    }
}
