use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::exif::ExifInfo;

/// Read EXIF info from a JPEG/JFIF file.
pub fn read_exif_jpg(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(&file);
    let exif_data = exif::Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| crate::error::AppError::Exif(e.to_string()))?;
    parse_exif_data(&exif_data)
}

/// Read EXIF info from a Fujifilm RAF raw file.
pub fn read_exif_raf(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    use crate::services::raw_decoders::raf_decoder::RafDecoder;

    let decoder = RafDecoder::new(file_path)
        .map_err(|e| crate::error::AppError::General(format!("Failed to decode RAF: {}", e)))?;

    let mut cursor = std::io::Cursor::new(&decoder.jpeg.data);
    let exif_data = exif::Reader::new()
        .read_from_container(&mut cursor)
        .map_err(|e| crate::error::AppError::Exif(e.to_string()))?;
    parse_exif_data(&exif_data)
}

/// Read EXIF info from an Adobe DNG raw file.
pub fn read_exif_dng(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    use crate::services::raw_decoders::dng_decoder::DngDecoder;

    let decoder = DngDecoder::new(file_path)
        .map_err(|e| crate::error::AppError::General(format!("Failed to decode DNG: {}", e)))?;

    if !decoder.preview.data.is_empty() {
        // 尝试从预览图中读取EXIF
        let mut cursor = std::io::Cursor::new(&decoder.preview.data);
        if let Ok(exif_data) = exif::Reader::new().read_from_container(&mut cursor) {
            return parse_exif_data(&exif_data);
        }
    }

    // 回退：直接从DNG文件读取EXIF（DNG基于TIFF格式，exif库可直接解析）
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(&file);
    let exif_data = exif::Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| crate::error::AppError::Exif(e.to_string()))?;
    parse_exif_data(&exif_data)
}

/// Extract [`ExifInfo`] from already-parsed EXIF data.
/// Shared by [`read_exif_jpg`] and [`read_exif_raf`].
pub(crate) fn parse_exif_data(exif_data: &exif::Exif) -> Result<ExifInfo, crate::error::AppError> {
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
            let lat_sign = if lat_ref.display_value().to_string().contains('S') { -1.0 } else { 1.0 };
            let lon_sign = if lon_ref.display_value().to_string().contains('W') { -1.0 } else { 1.0 };
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

    // Lens Model & Make
    let lens_make = exif_data
        .get_field(exif::Tag::LensMake, exif::In::PRIMARY)
        .and_then(|f| parse_ascii_field(&f.value));
    let lens_model = exif_data
        .get_field(exif::Tag::LensModel, exif::In::PRIMARY)
        .and_then(|f| parse_ascii_field(&f.value));
    info.lens_model = match (lens_make, lens_model) {
        (Some(make), Some(model)) => Some(format!("{}: {}", make, model)),
        (None, Some(model)) => Some(model),
        (Some(make), None) => Some(make),
        (None, None) => None,
    };

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
    if let Some(field) = exif_data.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
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
    if let Some(field) = exif_data.get_field(exif::Tag(exif::Context::Tiff, 0x4746), exif::In::PRIMARY) {
        if let exif::Value::Short(ref v) = field.value {
            if let Some(&val) = v.first() {
                if val <= 5 {
                    info.rating = Some(val as u8);
                }
            }
        }
    }

    Ok(info)
}

/// Convert GPS rational value to decimal degrees.
pub(crate) fn parse_gps_rational(value: &exif::Value) -> Option<f64> {
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

/// Parse ASCII EXIF value and return the first non-empty string.
/// Some cameras store multiple strings in ASCII fields (e.g., ["lens_name", "", "", ...]),
/// but we only want the actual lens name, not the empty strings.
pub(crate) fn parse_ascii_field(value: &exif::Value) -> Option<String> {
    if let exif::Value::Ascii(ref vec) = value {
        for s in vec {
            let trimmed = String::from_utf8_lossy(s).trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}
