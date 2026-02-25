use std::io::Cursor;
use std::path::Path;

use base64::Engine;
use image::imageops::FilterType;
use image::{ImageDecoder, ImageReader};
const THUMBNAIL_SIZE: u32 = 260;

pub fn generate_thumbnail(file_path: &Path) -> Result<String, crate::error::AppError> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let img = match ext.as_str() {
        "jpg" | "jpeg" | "png" => {
            // Use ImageReader to handle EXIF orientation
            // First get the decoder to read orientation, then decode and apply
            let reader = ImageReader::open(file_path)?;
            let mut decoder = reader.into_decoder()?;
            let orientation = decoder.orientation()?;
            let mut img = image::DynamicImage::from_decoder(decoder)?;
            img.apply_orientation(orientation);
            img
        }
        "dng" => {
            // For DNG files, try to read the embedded JPEG preview via EXIF
            // If that fails, return a placeholder
            match extract_dng_preview(file_path) {
                Ok(img) => img,
                Err(_) => {
                    // Return a simple gray placeholder for unsupported RAW
                    let img = image::DynamicImage::new_rgb8(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
                    img
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

pub fn read_full_image(file_path: &Path) -> Result<String, crate::error::AppError> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" => {
            let data = std::fs::read(file_path)?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
            Ok(b64)
        }
        "dng" => {
            // For DNG, convert to JPEG first
            let img = image::open(file_path)?;
            let mut buf = Vec::new();
            let mut cursor = Cursor::new(&mut buf);
            img.write_to(&mut cursor, image::ImageFormat::Jpeg)?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
            Ok(b64)
        }
        _ => Err(crate::error::AppError::General(format!(
            "Unsupported format: {}",
            ext
        ))),
    }
}
