use std::io::Cursor;
use std::path::Path;

use base64::Engine;
use image::imageops::FilterType;
use image::{ImageDecoder, ImageReader, DynamicImage};

use crate::services::image_cache::get_cache;

const THUMBNAIL_SIZE: u32 = 260;

/// 加载图片（用于缓存回调，处理 EXIF 方向）
fn load_image(file_path: &Path) -> Result<DynamicImage, crate::error::AppError> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let img = match ext.as_str() {
        "jpg" | "jpeg" | "png" => {
            // Use ImageReader to handle EXIF orientation
            let reader = ImageReader::open(file_path)?;
            let mut decoder = reader.into_decoder()?;
            let orientation = decoder.orientation()?;
            let mut img = DynamicImage::from_decoder(decoder)?;
            img.apply_orientation(orientation);
            img
        }
        "dng" => {
            match extract_dng_preview(file_path) {
                Ok(img) => img,
                Err(_) => {
                    // Return a simple gray placeholder for unsupported RAW
                    DynamicImage::new_rgb8(THUMBNAIL_SIZE, THUMBNAIL_SIZE)
                }
            }
        }
        "raf" => {
            match extract_raf_preview(file_path) {
                Ok(img) => img,
                Err(_) => {
                    DynamicImage::new_rgb8(THUMBNAIL_SIZE, THUMBNAIL_SIZE)
                }
            }
        }
        _ => {
            return Err(crate::error::AppError::General(format!(
                "Unsupported format: {}",
                ext
            )));
        }
    };

    Ok(img)
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

fn extract_dng_preview(file_path: &Path) -> Result<image::DynamicImage, crate::error::AppError> {
    // Try to read the DNG file as a regular image (some DNG files have embedded JPEGs)
    // The `image` crate can sometimes handle TIFF-based DNG files
    image::open(file_path).map_err(|e| crate::error::AppError::Image(e))
}

fn extract_raf_preview(file_path: &Path) -> Result<image::DynamicImage, crate::error::AppError> {
    // Use the RAF decoder to extract the embedded JPEG preview
    use crate::services::raw_decoders::raf_decoder::RafDecoder;

    let decoder = RafDecoder::new(file_path)
        .map_err(|e| crate::error::AppError::General(format!("Failed to decode RAF: {}", e)))?;

    // Load the JPEG data from the RAF file
    let jpeg_data = &decoder.jpeg.data;

    // Use image crate to load the JPEG data
    let img = image::load_from_memory(jpeg_data)
        .map_err(|e| crate::error::AppError::Image(e))?;

    Ok(img)
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
