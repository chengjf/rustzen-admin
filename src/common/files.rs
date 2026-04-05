use crate::common::error::ServiceError;

use axum::extract::Multipart;
use std::{fs::File, io::Write};
use uuid::Uuid;

const USER_AVATAR_DIR: &str = "uploads/avatars";
const USER_AVATAR_MAX_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy)]
enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    Webp,
}

impl ImageFormat {
    fn extension(self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
            ImageFormat::Gif => "gif",
            ImageFormat::Webp => "webp",
        }
    }
}

fn detect_image_format(data: &[u8]) -> Option<ImageFormat> {
    if data.len() >= 3 && data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageFormat::Jpeg);
    }

    if data.len() >= 8 && data.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(ImageFormat::Png);
    }

    if data.len() >= 6 && (data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a")) {
        return Some(ImageFormat::Gif);
    }

    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return Some(ImageFormat::Webp);
    }

    None
}

fn build_avatar_file_path(image_format: ImageFormat) -> String {
    format!("{}/{}.{}", USER_AVATAR_DIR, Uuid::new_v4(), image_format.extension())
}

fn validate_avatar_upload(data: &[u8]) -> Result<(ImageFormat, String), ServiceError> {
    if data.len() > USER_AVATAR_MAX_SIZE {
        return Err(ServiceError::InvalidOperation("File size must be less than 1MB".into()));
    }

    let image_format = detect_image_format(data).ok_or_else(|| {
        ServiceError::InvalidOperation(
            "Only JPEG, PNG, GIF and WebP image files are allowed".into(),
        )
    })?;

    Ok((image_format, build_avatar_file_path(image_format)))
}

fn write_avatar_file(file_path: &str, data: &[u8]) -> Result<(), ServiceError> {
    let mut file = File::create(file_path).map_err(|_| ServiceError::CreateAvatarFileFailed)?;
    file.write_all(data).map_err(|_| ServiceError::CreateAvatarFileFailed)?;
    Ok(())
}

/// 保存头像
pub async fn save_avatar(multipart: &mut Multipart) -> Result<String, ServiceError> {
    // 确保上传目录存在
    tokio::fs::create_dir_all(USER_AVATAR_DIR)
        .await
        .map_err(|_| ServiceError::CreateAvatarFolderFailed)?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ServiceError::InvalidOperation("Invalid multipart data".into()))?
    {
        // 读取文件数据
        let data = field
            .bytes()
            .await
            .map_err(|_| ServiceError::InvalidOperation("Failed to read file data".into()))?;

        let (_, file_path) = validate_avatar_upload(&data)?;

        // 保存文件
        write_avatar_file(&file_path, &data)?;

        let avatar_url = format!("/{}", file_path);
        tracing::info!("Avatar uploaded successfully: {}", avatar_url);

        return Ok(avatar_url);
    }

    Err(ServiceError::InvalidOperation("No file provided".into()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg_bytes() -> Vec<u8> {
        vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00]
    }

    fn png_bytes() -> Vec<u8> {
        vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00]
    }

    fn gif_bytes() -> Vec<u8> {
        b"GIF89a-test".to_vec()
    }

    fn webp_bytes() -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        bytes.extend_from_slice(b"WEBP");
        bytes
    }

    #[test]
    fn detects_supported_image_formats() {
        assert!(matches!(detect_image_format(&jpeg_bytes()), Some(ImageFormat::Jpeg)));
        assert!(matches!(detect_image_format(&png_bytes()), Some(ImageFormat::Png)));
        assert!(matches!(detect_image_format(&gif_bytes()), Some(ImageFormat::Gif)));
        assert!(matches!(detect_image_format(&webp_bytes()), Some(ImageFormat::Webp)));
    }

    #[test]
    fn rejects_unknown_image_format() {
        assert!(detect_image_format(b"not-an-image").is_none());
    }

    #[test]
    fn validate_avatar_upload_rejects_oversized_file() {
        let too_large = vec![0xFF; USER_AVATAR_MAX_SIZE + 1];
        let result = validate_avatar_upload(&too_large);
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[test]
    fn validate_avatar_upload_returns_server_generated_path_with_detected_extension() {
        let (_, path) = validate_avatar_upload(&png_bytes()).expect("png should be accepted");
        assert!(path.starts_with(USER_AVATAR_DIR));
        assert!(path.ends_with(".png"));
    }

    #[test]
    fn write_avatar_file_persists_bytes() {
        let temp_path = std::env::temp_dir().join(format!("rustzen-avatar-{}.png", Uuid::new_v4()));
        let file_path = temp_path.to_string_lossy().to_string();
        let bytes = png_bytes();

        write_avatar_file(&file_path, &bytes).expect("write should succeed");

        let written = std::fs::read(&temp_path).expect("file should exist");
        assert_eq!(written, bytes);

        std::fs::remove_file(temp_path).expect("cleanup should succeed");
    }
}
