use std::path::Path;

use little_exif::endian::Endian;
use little_exif::exif_tag::{ExifTag, ExifTagGroup};
use little_exif::exif_tag_format::ExifTagFormat;
use little_exif::metadata::Metadata;

/// Write a star rating to a JPEG file.
///
/// Sets both the `Rating` tag (0x4746) and the `RatingPercent` tag (0x4749)
/// for maximum compatibility with image viewers.
pub fn write_rating_jpg(file_path: &Path, rating: u8) -> Result<(), crate::error::AppError> {
    // Validate rating range (0-5)
    if rating > 5 {
        return Err(crate::error::AppError::General(
            "Rating must be between 0 and 5".to_string(),
        ));
    }

    if !file_path.exists() {
        return Err(crate::error::AppError::General(
            "File not found".to_string(),
        ));
    }

    let mut metadata = Metadata::new_from_path(file_path).map_err(|e| {
        crate::error::AppError::Exif(format!("Failed to read metadata: {}", e))
    })?;

    // Rating tag (0x4746) in IFD0
    let rating_bytes = (rating as u16).to_le_bytes().to_vec();
    let rating_tag = ExifTag::from_u16_with_data(
        0x4746,
        &ExifTagFormat::INT16U,
        &rating_bytes,
        &Endian::Little,
        &ExifTagGroup::IFD0,
    )
    .map_err(|e| crate::error::AppError::Exif(format!("Failed to create rating tag: {}", e)))?;
    metadata.set_tag(rating_tag);

    // RatingPercent tag (0x4749): convert 0-5 → 0, 20, 40, 60, 80, 100
    let percent = (rating as u16) * 20;
    let percent_bytes = percent.to_le_bytes().to_vec();
    let rating_percent_tag = ExifTag::from_u16_with_data(
        0x4749,
        &ExifTagFormat::INT16U,
        &percent_bytes,
        &Endian::Little,
        &ExifTagGroup::IFD0,
    )
    .map_err(|e| {
        crate::error::AppError::Exif(format!("Failed to create rating percent tag: {}", e))
    })?;
    metadata.set_tag(rating_percent_tag);

    metadata.write_to_file(file_path).map_err(|e| {
        crate::error::AppError::Exif(format!("Failed to write metadata: {}", e))
    })?;

    Ok(())
}
