use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Internal(err) => {
                tracing::error!(error = %err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "an internal error occurred".to_string(),
                )
            }
            AppError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                msg.clone(),
            ),
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });

        (status, axum::Json(body)).into_response()
    }
}
