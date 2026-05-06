use std::path::Path;

use crate::models::exif::ExifInfo;
use crate::services::raw_decoders::bmp_decoder::BmpDecoder;

// ── Read ──────────────────────────────────────────────────────────────────────

/// Read basic image info from a BMP file.
///
/// BMP format does not support EXIF metadata. This function extracts only
/// basic image dimensions (width/height) from the BMP header.
pub fn read_exif_bmp(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    let decoder = BmpDecoder::new(file_path)
        .map_err(|e| crate::error::AppError::General(format!("Failed to decode BMP: {}", e)))?;

    let (width, height) = decoder.get_image_dimensions();

    Ok(ExifInfo {
        image_width: Some(width),
        image_height: Some(height),
        ..Default::default()
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn test_read_exif_bmp() {
        let path = project_root().join("test_data").join("000010.BMP");
        let result = read_exif_bmp(&path);
        assert!(result.is_ok(), "Should be able to read BMP file: {:?}", result.err());

        let info = result.unwrap();
        // BMP should have dimensions
        assert!(info.image_width.is_some(), "BMP should have width");
        assert!(info.image_height.is_some(), "BMP should have height");
        assert!(info.image_width.unwrap() > 0, "BMP width should be > 0");
        assert!(info.image_height.unwrap() > 0, "BMP height should be > 0");
    }

    #[test]
    fn test_bmp_no_exif_fields() {
        let path = project_root().join("test_data").join("000010.BMP");
        let info = read_exif_bmp(&path).unwrap();

        // BMP has no EXIF support, so all camera-related fields should be None
        assert!(info.datetime.is_none(), "BMP should have no datetime");
        assert!(info.camera_make.is_none(), "BMP should have no camera make");
        assert!(info.camera_model.is_none(), "BMP should have no camera model");
        assert!(info.lens_model.is_none(), "BMP should have no lens model");
        assert!(info.focal_length.is_none(), "BMP should have no focal length");
        assert!(info.shutter_speed.is_none(), "BMP should have no shutter speed");
        assert!(info.aperture.is_none(), "BMP should have no aperture");
        assert!(info.iso.is_none(), "BMP should have no ISO");
        assert!(info.rating.is_none(), "BMP should have no rating");
        assert!(info.gps_latitude.is_none(), "BMP should have no GPS");
    }

    #[test]
    fn test_bmp_write_not_supported() {
        use crate::models::exif::ExifWriteRequest;

        let path = project_root().join("test_data").join("000010.BMP");
        let req = ExifWriteRequest {
            rating: Some(3),
            ..Default::default()
        };

        // Writing EXIF to BMP should fail via the module dispatcher
        let result = crate::services::exif_service::write_exif_fields(
            &[path.to_string_lossy().to_string()],
            &req,
        );
        assert!(result.is_err(), "EXIF write to BMP should be rejected");
    }
}
