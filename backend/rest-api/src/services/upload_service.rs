use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use uuid::Uuid;

use crate::errors::AppError;
use crate::traits;

/// Maximum upload size: 5 MB
const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

/// Max width after resize
const MAX_WIDTH: u32 = 800;

/// JPEG quality for output
const JPEG_QUALITY: u8 = 80;

/// Allowed MIME types
const ALLOWED_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];

// ── R2 Implementation ───────────────────────────────────────────────

pub struct R2UploadService {
    client: aws_sdk_s3::Client,
    bucket: String,
    public_url: String,
}

impl R2UploadService {
    pub async fn new(
        account_id: &str,
        access_key_id: &str,
        secret_access_key: &str,
        bucket: String,
        public_url: String,
    ) -> Self {
        let endpoint_url = format!("https://{account_id}.r2.cloudflarestorage.com");

        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "r2",
        );

        let config = aws_sdk_s3::Config::builder()
            .region(aws_sdk_s3::config::Region::new("auto"))
            .endpoint_url(&endpoint_url)
            .credentials_provider(credentials)
            .force_path_style(true)
            .behavior_version_latest()
            .build();

        let client = aws_sdk_s3::Client::from_conf(config);

        Self {
            client,
            bucket,
            public_url,
        }
    }
}

#[async_trait]
impl traits::UploadService for R2UploadService {
    async fn upload_image(&self, data: &[u8], content_type: &str) -> Result<String, AppError> {
        // Validate size
        if data.len() > MAX_UPLOAD_BYTES {
            return Err(AppError::BadRequest(format!(
                "image too large: {} bytes (max {})",
                data.len(),
                MAX_UPLOAD_BYTES
            )));
        }

        // Validate content type
        if !ALLOWED_TYPES.contains(&content_type) {
            return Err(AppError::BadRequest(format!(
                "unsupported image type: {content_type}. Allowed: {}",
                ALLOWED_TYPES.join(", ")
            )));
        }

        // Decode, resize, and re-encode as JPEG
        let processed = tokio::task::spawn_blocking({
            let data = data.to_vec();
            move || process_image(&data)
        })
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .map_err(AppError::Internal)?;

        // Upload to R2
        let key = format!("uploads/{}.jpg", Uuid::new_v4());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(processed))
            .content_type("image/jpeg")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("R2 upload failed: {e}")))?;

        let url = format!("{}/{}", self.public_url.trim_end_matches('/'), key);
        Ok(url)
    }

    async fn delete_image(&self, url: &str) -> Result<(), AppError> {
        // Extract key from URL
        let key = url
            .strip_prefix(&self.public_url)
            .map(|k| k.trim_start_matches('/'))
            .unwrap_or(url);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("R2 delete failed: {e}")))?;

        Ok(())
    }
}

/// Process image: decode, resize if needed, encode as JPEG.
fn process_image(data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let img = image::load_from_memory(data)?;

    // Resize if width > MAX_WIDTH
    let img = if img.width() > MAX_WIDTH {
        img.resize(MAX_WIDTH, u32::MAX, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // Encode as JPEG
    let mut output = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, JPEG_QUALITY);
    img.write_with_encoder(encoder)?;

    Ok(output)
}

// ── Noop Implementation (for tests) ─────────────────────────────────

pub struct NoopUploadService;

#[async_trait]
impl traits::UploadService for NoopUploadService {
    async fn upload_image(&self, _data: &[u8], _content_type: &str) -> Result<String, AppError> {
        Ok(format!(
            "https://test-cdn.offrii.com/uploads/{}.jpg",
            Uuid::new_v4()
        ))
    }

    async fn delete_image(&self, _url: &str) -> Result<(), AppError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_valid_jpeg() {
        // Create a minimal 2x2 JPEG in memory
        let mut buf = Vec::new();
        let img = image::RgbImage::from_pixel(100, 100, image::Rgb([255, 0, 0]));
        let encoder = image::codecs::jpeg::JpegEncoder::new(&mut buf);
        image::DynamicImage::ImageRgb8(img)
            .write_with_encoder(encoder)
            .unwrap();

        let result = process_image(&buf);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn process_resizes_large_image() {
        let mut buf = Vec::new();
        let img = image::RgbImage::from_pixel(2000, 1500, image::Rgb([0, 128, 255]));
        let encoder = image::codecs::jpeg::JpegEncoder::new(&mut buf);
        image::DynamicImage::ImageRgb8(img)
            .write_with_encoder(encoder)
            .unwrap();

        let result = process_image(&buf).unwrap();

        // Decode result and check it was resized
        let output_img = image::load_from_memory(&result).unwrap();
        assert_eq!(output_img.width(), MAX_WIDTH);
        assert!(output_img.height() < 1500);
    }

    #[test]
    fn process_rejects_invalid_data() {
        let result = process_image(b"this is not an image");
        assert!(result.is_err());
    }
}
