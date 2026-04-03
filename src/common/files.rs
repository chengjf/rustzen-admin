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

        // 验证文件大小
        if data.len() > USER_AVATAR_MAX_SIZE {
            return Err(
                ServiceError::InvalidOperation("File size must be less than 1MB".into()).into()
            );
        }

        let image_format = detect_image_format(&data).ok_or_else(|| {
            ServiceError::InvalidOperation(
                "Only JPEG, PNG, GIF and WebP image files are allowed".into(),
            )
        })?;

        // Ignore client-provided filename/extensions and persist using server-detected format.
        let file_path = format!(
            "{}/{}.{}",
            USER_AVATAR_DIR,
            Uuid::new_v4(),
            image_format.extension()
        );

        // 保存文件
        let mut file =
            File::create(&file_path).map_err(|_| ServiceError::CreateAvatarFileFailed)?;
        file.write_all(&data).map_err(|_| ServiceError::CreateAvatarFileFailed)?;

        let avatar_url = format!("/{}", file_path);
        tracing::info!("Avatar uploaded successfully: {}", avatar_url);

        return Ok(avatar_url);
    }

    Err(ServiceError::InvalidOperation("No file provided".into()).into())
}
