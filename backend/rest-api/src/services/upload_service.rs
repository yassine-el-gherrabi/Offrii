use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use uuid::Uuid;

use crate::errors::AppError;
use crate::traits;

/// Maximum upload size: 5 MB
const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

/// WebP lossy encoding quality (0-100).
const WEBP_QUALITY: f32 = 75.0;

/// Allowed input MIME types
const ALLOWED_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/heic",
    "image/heif",
];

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
    async fn upload_image(
        &self,
        data: &[u8],
        content_type: &str,
        upload_type: &str,
    ) -> Result<String, AppError> {
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

        // Decode, apply EXIF orientation, resize based on type, and re-encode as WebP
        let processed = tokio::task::spawn_blocking({
            let data = data.to_vec();
            let upload_type = upload_type.to_string();
            move || process_image(&data, &upload_type)
        })
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .map_err(AppError::Internal)?;

        // Upload to R2 with type-based prefix
        let prefix = match upload_type {
            "avatar" => "avatars",
            "circle" => "circles",
            _ => "items",
        };
        let key = format!("{prefix}/{}.webp", Uuid::new_v4());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(processed))
            .content_type("image/webp")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("R2 upload failed: {e}")))?;

        let url = format!("{}/{}", self.public_url.trim_end_matches('/'), key);
        Ok(url)
    }

    async fn delete_image(&self, url: &str) -> Result<(), AppError> {
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

/// Read EXIF orientation tag from raw image bytes.
/// Returns orientation value 1-8 (1 = normal). Falls back to 1 on any error.
fn read_exif_orientation(data: &[u8]) -> u16 {
    let mut cursor = std::io::Cursor::new(data);
    let Ok(exif) = exif::Reader::new().read_from_container(&mut cursor) else {
        return 1;
    };
    exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .map(|v| v as u16)
        .unwrap_or(1)
}

/// Apply EXIF orientation transform to decoded image.
fn apply_orientation(img: image::DynamicImage, orientation: u16) -> image::DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate90().flipv(),
        8 => img.rotate270(),
        _ => img, // 1 or unknown = no transform
    }
}

/// Process image: decode, apply EXIF orientation, resize based on type, encode as WebP.
fn process_image(data: &[u8], upload_type: &str) -> Result<Vec<u8>, anyhow::Error> {
    // Read EXIF orientation before decoding (decoding strips EXIF)
    let orientation = read_exif_orientation(data);

    let img = image::load_from_memory(data)?;

    // Apply EXIF orientation (must be before crop/resize as it changes dimensions)
    let img = apply_orientation(img, orientation);

    let img = match upload_type {
        "avatar" | "circle" => {
            // Crop to square (center crop), then resize to 400px
            let min_dim = img.width().min(img.height());
            let x = (img.width() - min_dim) / 2;
            let y = (img.height() - min_dim) / 2;
            let cropped = img.crop_imm(x, y, min_dim, min_dim);
            if min_dim > 400 {
                cropped.resize(400, 400, image::imageops::FilterType::Lanczos3)
            } else {
                cropped
            }
        }
        _ => {
            // Item: resize to max 800px width
            if img.width() > 800 {
                img.resize(800, u32::MAX, image::imageops::FilterType::Lanczos3)
            } else {
                img
            }
        }
    };

    // Encode as WebP lossy
    let encoder = webp::Encoder::from_image(&img)
        .map_err(|e| anyhow::anyhow!("WebP encoding failed: {e}"))?;
    let webp_data = encoder.encode(WEBP_QUALITY);

    Ok(webp_data.to_vec())
}

// ── Noop Implementation (for dev/tests) ─────────────────────────────

pub struct NoopUploadService;

#[async_trait]
impl traits::UploadService for NoopUploadService {
    async fn upload_image(
        &self,
        _data: &[u8],
        _content_type: &str,
        upload_type: &str,
    ) -> Result<String, AppError> {
        let prefix = match upload_type {
            "avatar" => "avatars",
            "circle" => "circles",
            _ => "items",
        };
        Ok(format!(
            "https://test-cdn.offrii.com/{}/{}.webp",
            prefix,
            Uuid::new_v4()
        ))
    }

    async fn delete_image(&self, _url: &str) -> Result<(), AppError> {
        Ok(())
    }
}
