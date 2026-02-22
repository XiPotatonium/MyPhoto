use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::exif::ExifInfo;

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
    // Note: kamadak-exif may not support XMP rating natively.
    // We'll handle rating separately if needed.

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

pub fn write_rating(_file_path: &Path, _rating: u8) -> Result<(), crate::error::AppError> {
    // Rating writing requires modifying EXIF data.
    // kamadak-exif is read-only. For a full implementation, we would use
    // little_exif or img_parts crate. For now, this is a placeholder.
    // TODO: Implement EXIF rating write with a suitable crate
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
