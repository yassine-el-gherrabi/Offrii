use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::Multipart;
use serde::Serialize;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;

/// Maximum upload size: 5 MB
const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/image", post(upload_image))
}

#[tracing::instrument(skip(state, multipart))]
async fn upload_image(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    // Extract the "image" field from multipart form
    let mut image_data: Option<(Vec<u8>, String)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("invalid multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("failed to read image: {e}")))?;

            if bytes.len() > MAX_UPLOAD_BYTES {
                return Err(AppError::BadRequest(format!(
                    "image too large: {} bytes (max {MAX_UPLOAD_BYTES})",
                    bytes.len()
                )));
            }

            image_data = Some((bytes.to_vec(), content_type));
            break;
        }
    }

    let (data, content_type) =
        image_data.ok_or_else(|| AppError::BadRequest("missing 'image' field".into()))?;

    let url = state.uploads.upload_image(&data, &content_type).await?;

    Ok((StatusCode::CREATED, Json(UploadResponse { url })))
}
