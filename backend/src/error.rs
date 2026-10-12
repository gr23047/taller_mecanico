use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Conflict(String),
    Internal(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            ApiError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, m),
            ApiError::Internal(m) => {
                // El detalle se queda en el log; al cliente no se le expone
                tracing::error!("error interno: {m}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
