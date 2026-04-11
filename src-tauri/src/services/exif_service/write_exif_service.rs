use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

use exif::experimental::Writer as ExifWriter;
use exif::{Field, In, Rational, Tag, Value};

use crate::models::exif::ExifWriteRequest;
use super::write_gps_service::write_jpeg_with_exif;

/// Write multiple EXIF fields to a JPEG file.
///
/// Uses a merge strategy: reads existing EXIF, replaces the requested fields,
/// and writes the result back. Fields set to `None` in the request are left
/// unchanged (their existing values are preserved).
pub fn write_exif_fields_jpg(
    file_path: &Path,
    req: &ExifWriteRequest,
) -> Result<(), crate::error::AppError> {
    if !file_path.exists() {
        return Err(crate::error::AppError::General(format!(
            "File not found: {}",
            file_path.display()
        )));
    }

    // Collect the set of tags that will be overwritten so we can strip them
    // from the existing EXIF before appending the new values.
    let mut tags_to_replace: Vec<Tag> = Vec::new();

    if req.datetime.is_some() {
        tags_to_replace.push(Tag::DateTimeOriginal);
        tags_to_replace.push(Tag::DateTime);
    }
    if req.camera_model.is_some() {
        tags_to_replace.push(Tag::Model);
    }
    if req.lens_model.is_some() {
        tags_to_replace.push(Tag::LensModel);
    }
    if req.focal_length.is_some() {
        tags_to_replace.push(Tag::FocalLength);
    }
    if req.shutter_speed.is_some() {
        tags_to_replace.push(Tag::ExposureTime);
    }
    if req.aperture.is_some() {
        tags_to_replace.push(Tag::FNumber);
    }
    if req.iso.is_some() {
        tags_to_replace.push(Tag::PhotographicSensitivity);
    }
    if req.gps_latitude.is_some() || req.gps_longitude.is_some() {
        tags_to_replace.push(Tag::GPSVersionID);
        tags_to_replace.push(Tag::GPSLatitudeRef);
        tags_to_replace.push(Tag::GPSLatitude);
        tags_to_replace.push(Tag::GPSLongitudeRef);
        tags_to_replace.push(Tag::GPSLongitude);
    }

    // Build new fields to write
    let mut new_fields: Vec<Field> = Vec::new();

    if let Some(ref dt) = req.datetime {
        let ascii_val = Value::Ascii(vec![dt.as_bytes().to_vec()]);
        new_fields.push(Field {
            tag: Tag::DateTimeOriginal,
            ifd_num: In::PRIMARY,
            value: ascii_val.clone(),
        });
        new_fields.push(Field {
            tag: Tag::DateTime,
            ifd_num: In::PRIMARY,
            value: ascii_val,
        });
    }

    if let Some(ref model) = req.camera_model {
        new_fields.push(Field {
            tag: Tag::Model,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![model.as_bytes().to_vec()]),
        });
    }

    if let Some(ref lens) = req.lens_model {
        new_fields.push(Field {
            tag: Tag::LensModel,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![lens.as_bytes().to_vec()]),
        });
    }

    if let Some(fl) = req.focal_length {
        // Store focal length as rational with denominator 1000 for sub-mm precision
        let num = (fl * 1000.0).round() as u32;
        new_fields.push(Field {
            tag: Tag::FocalLength,
            ifd_num: In::PRIMARY,
            value: Value::Rational(vec![Rational { num, denom: 1000 }]),
        });
    }

    if let Some(ref ss) = req.shutter_speed {
        let (num, denom) = parse_shutter_speed(ss).ok_or_else(|| {
            crate::error::AppError::General(format!(
                "Invalid shutter speed format: '{}'. Use '1/500' or '2'.",
                ss
            ))
        })?;
        new_fields.push(Field {
            tag: Tag::ExposureTime,
            ifd_num: In::PRIMARY,
            value: Value::Rational(vec![Rational { num, denom }]),
        });
    }

    if let Some(ap) = req.aperture {
        // Store aperture as rational with denominator 100
        let num = (ap * 100.0).round() as u32;
        new_fields.push(Field {
            tag: Tag::FNumber,
            ifd_num: In::PRIMARY,
            value: Value::Rational(vec![Rational { num, denom: 100 }]),
        });
    }

    if let Some(iso) = req.iso {
        new_fields.push(Field {
            tag: Tag::PhotographicSensitivity,
            ifd_num: In::PRIMARY,
            value: Value::Short(vec![iso.min(65535) as u16]),
        });
    }

    if let (Some(lat), Some(lon)) = (req.gps_latitude, req.gps_longitude) {
        let lat_ref = if lat >= 0.0 { "N" } else { "S" };
        let lon_ref = if lon >= 0.0 { "E" } else { "W" };
        let lat_dms = decimal_degrees_to_dms(lat.abs());
        let lon_dms = decimal_degrees_to_dms(lon.abs());

        new_fields.push(Field {
            tag: Tag::GPSVersionID,
            ifd_num: In::PRIMARY,
            value: Value::Byte(vec![2, 3, 0, 0]),
        });
        new_fields.push(Field {
            tag: Tag::GPSLatitudeRef,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![lat_ref.as_bytes().to_vec()]),
        });
        new_fields.push(Field {
            tag: Tag::GPSLatitude,
            ifd_num: In::PRIMARY,
            value: Value::Rational(lat_dms),
        });
        new_fields.push(Field {
            tag: Tag::GPSLongitudeRef,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![lon_ref.as_bytes().to_vec()]),
        });
        new_fields.push(Field {
            tag: Tag::GPSLongitude,
            ifd_num: In::PRIMARY,
            value: Value::Rational(lon_dms),
        });
    }

    let original_data = std::fs::read(file_path)?;

    // Merge with existing EXIF
    let file = File::open(file_path)?;
    let mut bufreader = BufReader::new(&file);
    let exif_reader = exif::Reader::new();

    // Collect existing fields that won't be overwritten
    let mut kept_fields: Vec<Field> = Vec::new();

    match exif_reader.read_from_container(&mut bufreader) {
        Ok(existing_exif) => {
            kept_fields = existing_exif
                .fields()
                .filter(|f| !tags_to_replace.contains(&f.tag))
                .cloned()
                .collect();
        }
        Err(_) => {
            // No existing EXIF — start fresh
        }
    }

    let mut merged_writer = ExifWriter::new();
    for field in &kept_fields {
        merged_writer.push_field(field);
    }

    for field in &new_fields {
        merged_writer.push_field(field);
    }

    let mut buf = Cursor::new(Vec::new());
    merged_writer
        .write(&mut buf, false)
        .map_err(|e| crate::error::AppError::Exif(format!("Failed to write EXIF: {}", e)))?;

    let exif_data = buf.into_inner();
    write_jpeg_with_exif(file_path, &original_data, &exif_data)?;

    Ok(())
}

/// Parse a shutter speed string into (numerator, denominator).
///
/// Accepts:
/// - `"1/500"` → `(1, 500)`
/// - `"2"` or `"2.0"` → `(2, 1)`
/// - `"0.5"` → `(1, 2)`
fn parse_shutter_speed(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    if let Some(slash_pos) = s.find('/') {
        let num: u32 = s[..slash_pos].trim().parse().ok()?;
        let denom: u32 = s[slash_pos + 1..].trim().parse().ok()?;
        if denom == 0 {
            return None;
        }
        Some((num, denom))
    } else {
        // Parse as decimal seconds
        let secs: f64 = s.parse().ok()?;
        if secs <= 0.0 {
            return None;
        }
        if secs >= 1.0 {
            Some((secs.round() as u32, 1))
        } else {
            // e.g. 0.5 → 1/2
            let denom = (1.0 / secs).round() as u32;
            Some((1, denom))
        }
    }
}

/// Convert absolute decimal degrees to 3 DMS RATIONAL values.
fn decimal_degrees_to_dms(decimal: f64) -> Vec<Rational> {
    let degrees = decimal.floor();
    let minutes_full = (decimal - degrees) * 60.0;
    let minutes = minutes_full.floor();
    let seconds = (minutes_full - minutes) * 60.0;
    let sec_numerator = (seconds * 1_000_000.0).round() as u32;

    vec![
        Rational { num: degrees as u32, denom: 1 },
        Rational { num: minutes as u32, denom: 1 },
        Rational { num: sec_numerator, denom: 1_000_000 },
    ]
}
