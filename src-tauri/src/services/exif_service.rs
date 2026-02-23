use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::exif::ExifInfo;

// little_exif imports for writing EXIF data
use little_exif::metadata::Metadata;
use little_exif::exif_tag::{ExifTag, ExifTagGroup};
use little_exif::exif_tag_format::ExifTagFormat;
use little_exif::endian::Endian;

pub fn read_exif(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(&file);

    let exif_reader = exif::Reader::new();
    let exif_data = exif_reader
        .read_from_container(&mut reader)
        .map_err(|e| crate::error::AppError::Exif(e.to_string()))?;

    let mut info = ExifInfo::default();

    // DateTime
    if let Some(field) = exif_data.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        info.datetime = Some(field.display_value().to_string());
    } else if let Some(field) = exif_data.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
        info.datetime = Some(field.display_value().to_string());
    }

    // GPS
    if let (Some(lat), Some(lat_ref), Some(lon), Some(lon_ref)) = (
        exif_data.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY),
        exif_data.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY),
        exif_data.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY),
        exif_data.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY),
    ) {
        if let (Some(lat_val), Some(lon_val)) = (
            parse_gps_rational(&lat.value),
            parse_gps_rational(&lon.value),
        ) {
            let lat_sign = if lat_ref.display_value().to_string().contains('S') {
                -1.0
            } else {
                1.0
            };
            let lon_sign = if lon_ref.display_value().to_string().contains('W') {
                -1.0
            } else {
                1.0
            };
            info.gps_latitude = Some(lat_val * lat_sign);
            info.gps_longitude = Some(lon_val * lon_sign);
        }
    }

    // Camera Make
    if let Some(field) = exif_data.get_field(exif::Tag::Make, exif::In::PRIMARY) {
        info.camera_make = Some(field.display_value().to_string().trim_matches('"').to_string());
    }

    // Camera Model
    if let Some(field) = exif_data.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        info.camera_model = Some(field.display_value().to_string().trim_matches('"').to_string());
    }

    // Lens Model
    if let Some(field) = exif_data.get_field(exif::Tag::LensModel, exif::In::PRIMARY) {
        info.lens_model = Some(field.display_value().to_string().trim_matches('"').to_string());
    }

    // Focal Length
    if let Some(field) = exif_data.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        if let exif::Value::Rational(ref v) = field.value {
            if let Some(r) = v.first() {
                info.focal_length = Some(r.to_f64() as f32);
            }
        }
    }

    // Shutter Speed (ExposureTime)
    if let Some(field) = exif_data.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        info.shutter_speed = Some(field.display_value().to_string());
    }

    // Aperture (FNumber)
    if let Some(field) = exif_data.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        if let exif::Value::Rational(ref v) = field.value {
            if let Some(r) = v.first() {
                info.aperture = Some(r.to_f64() as f32);
            }
        }
    }

    // ISO
    if let Some(field) = exif_data.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY)
    {
        if let exif::Value::Short(ref v) = field.value {
            if let Some(&val) = v.first() {
                info.iso = Some(val as u32);
            }
        } else if let exif::Value::Long(ref v) = field.value {
            if let Some(&val) = v.first() {
                info.iso = Some(val);
            }
        }
    }

    // Image dimensions
    if let Some(field) = exif_data.get_field(exif::Tag::PixelXDimension, exif::In::PRIMARY) {
        if let exif::Value::Long(ref v) = field.value {
            info.image_width = v.first().copied();
        } else if let exif::Value::Short(ref v) = field.value {
            info.image_width = v.first().map(|&x| x as u32);
        }
    }
    if let Some(field) = exif_data.get_field(exif::Tag::PixelYDimension, exif::In::PRIMARY) {
        if let exif::Value::Long(ref v) = field.value {
            info.image_height = v.first().copied();
        } else if let exif::Value::Short(ref v) = field.value {
            info.image_height = v.first().map(|&x| x as u32);
        }
    }

    // Rating (XMP Rating tag - 0x4746)
    // kamadak-exif supports reading custom tags using Tag::Unknown
    if let Some(field) = exif_data.get_field(exif::Tag(exif::Context::Tiff, 0x4746), exif::In::PRIMARY) {
        if let exif::Value::Short(ref v) = field.value {
            if let Some(&val) = v.first() {
                // Rating is 0-5
                if val <= 5 {
                    info.rating = Some(val as u8);
                }
            }
        }
    }

    // for field in exif_data.fields() {
    //     println!("{:?}: {:?}", field.tag.number(), field.display_value().to_string());
    // }

    Ok(info)
}

fn parse_gps_rational(value: &exif::Value) -> Option<f64> {
    if let exif::Value::Rational(ref v) = value {
        if v.len() >= 3 {
            let degrees = v[0].to_f64();
            let minutes = v[1].to_f64();
            let seconds = v[2].to_f64();
            return Some(degrees + minutes / 60.0 + seconds / 3600.0);
        }
    }
    None
}

pub fn write_rating(file_path: &Path, rating: u8) -> Result<(), crate::error::AppError> {
    // Validate rating range (0-5)
    if rating > 5 {
        return Err(crate::error::AppError::General(
            "Rating must be between 0 and 5".to_string(),
        ));
    }

    // Check if file exists
    if !file_path.exists() {
        return Err(crate::error::AppError::General(
            "File not found".to_string(),
        ));
    }

    // Load existing metadata or create new
    let mut metadata = Metadata::new_from_path(file_path).map_err(|e| {
        crate::error::AppError::Exif(format!("Failed to read metadata: {}", e))
    })?;

    // Set the Rating tag (0x4746 in IFD0)
    // Rating 0 means "not rated" or "rejected" depending on the application
    // Using from_u16_with_data to create the tag since Rating is not a predefined variant
    // Convert u16 rating to little-endian bytes
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

    // Also set the RatingPercent tag (0x4749) for compatibility
    // Convert 0-5 rating to percentage (0, 20, 40, 60, 80, 100)
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

    // Write metadata back to file
    metadata.write_to_file(file_path).map_err(|e| {
        crate::error::AppError::Exif(format!("Failed to write metadata: {}", e))
    })?;

    Ok(())
}

pub fn batch_write_gps(
    _file_paths: &[String],
    _latitude: f64,
    _longitude: f64,
) -> Result<(), crate::error::AppError> {
    // GPS writing requires modifying EXIF data.
    // This is a placeholder for the same reason as write_rating.
    // TODO: Implement GPS EXIF write
    Ok(())
}
