use std::io::Cursor;
use std::path::Path;

use base64::Engine;
use image::imageops::FilterType;
use image::{ImageDecoder, ImageReader, DynamicImage};

use crate::services::image_cache::get_cache;

const THUMBNAIL_SIZE: u32 = 260;

/// 从图片字节数据加载图片（处理 EXIF 方向）
/// 可被多种格式的解码路径复用（如 JPG/JPEG/PNG/BMP/TIFF 直接读取，RAF 提取内嵌 JPEG 后调用此函数）
fn decode_image_bytes(image_bytes: &[u8]) -> Result<DynamicImage, crate::error::AppError> {
    let cursor = Cursor::new(image_bytes);
    let reader = ImageReader::new(cursor).with_guessed_format()?;
    let mut decoder = reader.into_decoder()?;
    // orientation() 可能对某些格式(BMP等)不适用，使用默认值避免错误中断
    let orientation = decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    Ok(img)
}

/// 从标准图片文件（JPG/PNG/BMP/TIFF 等 image crate 支持的格式）加载
fn load_standard_image(file_path: &Path) -> Result<DynamicImage, crate::error::AppError> {
    let bytes = std::fs::read(file_path)?;
    decode_image_bytes(&bytes)
}

/// 从 RAF 文件加载：提取内嵌 JPEG 后解码
fn load_raf_image(file_path: &Path) -> Result<DynamicImage, crate::error::AppError> {
    use crate::services::raw_decoders::raf_decoder::RafDecoder;
    let decoder = RafDecoder::new(file_path)
        .map_err(|e| crate::error::AppError::General(format!("Failed to decode RAF: {}", e)))?;
    decode_image_bytes(&decoder.jpeg.data)
}

/// 从 DNG 文件加载：提取内嵌预览图后解码
fn load_dng_image(file_path: &Path) -> Result<DynamicImage, crate::error::AppError> {
    use crate::services::raw_decoders::dng_decoder::DngDecoder;
    let decoder = DngDecoder::new(file_path)
        .map_err(|e| crate::error::AppError::General(format!("Failed to decode DNG: {}", e)))?;
    if decoder.preview.data.is_empty() {
        // 如果没有预览图，尝试用标准方式加载
        return load_standard_image(file_path);
    }
    decode_image_bytes(&decoder.preview.data)
}

/// 加载图片（用于缓存回调）
/// 根据文件扩展名分发到对应的格式解码器，便于未来扩展新格式
fn load_image(file_path: &Path) -> Result<DynamicImage, crate::error::AppError> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "tif" | "tiff" | "bmp" => load_standard_image(file_path),
        "raf" => load_raf_image(file_path),
        "dng" => load_dng_image(file_path),
        _ => Err(crate::error::AppError::General(format!(
            "Unsupported format: {}",
            ext
        ))),
    }
}

pub fn generate_thumbnail(file_path: &Path) -> Result<String, crate::error::AppError> {
    // 使用缓存获取或加载图片
    let cache = get_cache();
    let img = cache.get_or_load(file_path, load_image)?;

    let thumb = img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Triangle);

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    thumb.write_to(&mut cursor, image::ImageFormat::Jpeg)?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(b64)
}

pub fn read_full_image(file_path: &Path) -> Result<String, crate::error::AppError> {
    // 使用缓存获取或加载图片（复用 load_original_image 处理 EXIF 方向）
    let cache = get_cache();
    let img = cache.get_or_load(file_path, load_image)?;

    // 编码为 base64
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, image::ImageFormat::Jpeg)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);

    Ok(b64)
}
