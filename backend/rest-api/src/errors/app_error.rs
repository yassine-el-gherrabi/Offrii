use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("not found: {0}")]
    NotFound(String),
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
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
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

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;
    use serde_json::Value;

    use super::*;

    async fn extract_response(err: AppError) -> (StatusCode, Value) {
        let resp = err.into_response();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn internal_returns_500_with_generic_message() {
        let (status, body) =
            extract_response(AppError::Internal(anyhow::anyhow!("secret db info"))).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "INTERNAL_ERROR");
        assert_eq!(body["error"]["message"], "an internal error occurred");
        // Must NOT leak the real error message
        assert!(
            !body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("secret")
        );
    }

    #[tokio::test]
    async fn unauthorized_returns_401() {
        let (status, body) = extract_response(AppError::Unauthorized("invalid token".into())).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "UNAUTHORIZED");
        assert_eq!(body["error"]["message"], "invalid token");
    }

    #[tokio::test]
    async fn conflict_returns_409() {
        let (status, body) = extract_response(AppError::Conflict("email taken".into())).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], "CONFLICT");
        assert_eq!(body["error"]["message"], "email taken");
    }

    #[tokio::test]
    async fn bad_request_returns_400() {
        let (status, body) = extract_response(AppError::BadRequest("invalid email".into())).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "BAD_REQUEST");
        assert_eq!(body["error"]["message"], "invalid email");
    }

    #[tokio::test]
    async fn service_unavailable_returns_503() {
        let (status, body) =
            extract_response(AppError::ServiceUnavailable("redis down".into())).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"]["code"], "SERVICE_UNAVAILABLE");
        assert_eq!(body["error"]["message"], "redis down");
    }
}
