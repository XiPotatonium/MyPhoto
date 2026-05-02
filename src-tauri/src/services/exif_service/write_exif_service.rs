use std::io::Cursor;
use std::path::Path;

use byteorder::{BigEndian, ByteOrder};
use exif::experimental::Writer as ExifWriter;
use exif::{Context, Field, In, Rational, Tag, Value};

use crate::models::exif::ExifWriteRequest;

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Build the list of EXIF tags to replace and the new fields from a write request.
///
/// Shared by both JPEG and RAF write paths. Pure data conversion, no I/O.
fn build_exif_changes(
    req: &ExifWriteRequest,
) -> Result<(Vec<Tag>, Vec<Field>), crate::error::AppError> {
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
    if req.rating.is_some() {
        tags_to_replace.push(Tag(Context::Tiff, 0x4746)); // Rating
        tags_to_replace.push(Tag(Context::Tiff, 0x4749)); // RatingPercent
    }

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

    if let Some(rating) = req.rating {
        if rating > 5 {
            return Err(crate::error::AppError::General(
                "Rating must be between 0 and 5".to_string(),
            ));
        }
        // Rating tag (0x4746) in IFD0
        new_fields.push(Field {
            tag: Tag(Context::Tiff, 0x4746),
            ifd_num: In::PRIMARY,
            value: Value::Short(vec![rating as u16]),
        });
        // RatingPercent tag (0x4749): 0-5 → 0, 20, 40, 60, 80, 100
        new_fields.push(Field {
            tag: Tag(Context::Tiff, 0x4749),
            ifd_num: In::PRIMARY,
            value: Value::Short(vec![(rating as u16) * 20]),
        });
    }

    Ok((tags_to_replace, new_fields))
}

/// Read existing EXIF from JPEG data, merge with new fields, and serialize.
///
/// Returns the raw EXIF bytes (TIFF format) ready to be wrapped in an APP1 segment.
fn build_merged_exif_bytes(
    jpeg_data: &[u8],
    tags_to_replace: &[Tag],
    new_fields: &[Field],
) -> Result<Vec<u8>, crate::error::AppError> {
    let mut cursor = Cursor::new(jpeg_data);
    let exif_reader = exif::Reader::new();

    let mut kept_fields: Vec<Field> = Vec::new();
    match exif_reader.read_from_container(&mut cursor) {
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
    for field in new_fields {
        merged_writer.push_field(field);
    }

    let mut buf = Cursor::new(Vec::new());
    merged_writer
        .write(&mut buf, false)
        .map_err(|e| crate::error::AppError::Exif(format!("Failed to write EXIF: {}", e)))?;

    Ok(buf.into_inner())
}

/// Replace or insert the APP1 EXIF segment in JPEG data, returning new JPEG bytes.
fn build_jpeg_with_exif(
    original_jpeg: &[u8],
    exif_data: &[u8],
) -> Result<Vec<u8>, crate::error::AppError> {
    const SOI: [u8; 2] = [0xFF, 0xD8];
    const APP1: u8 = 0xE1;
    const EXIF_HEADER: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00]; // "Exif\0\0"

    if original_jpeg.len() < 2 || original_jpeg[0..2] != SOI {
        return Err(crate::error::AppError::General(
            "Not a valid JPEG file".to_string(),
        ));
    }

    let mut output = Vec::new();
    output.extend_from_slice(&SOI);

    // Build the new APP1 EXIF segment
    let exif_segment_size = 2 + 6 + exif_data.len(); // size field (2) + "Exif\0\0" (6) + data
    if exif_segment_size > 0xFFFF {
        return Err(crate::error::AppError::General(
            "EXIF data too large".to_string(),
        ));
    }

    output.push(0xFF);
    output.push(APP1);
    output.extend_from_slice(&(exif_segment_size as u16).to_be_bytes());
    output.extend_from_slice(&EXIF_HEADER);
    output.extend_from_slice(exif_data);

    // Parse the rest of the original JPEG, skipping existing EXIF APP1 segment
    let mut pos = 2; // Skip SOI

    while pos + 1 < original_jpeg.len() {
        if original_jpeg[pos] != 0xFF {
            break;
        }

        let marker = original_jpeg[pos + 1];
        pos += 2;

        // Skip existing APP1 EXIF segment
        if marker == APP1 && pos + 2 <= original_jpeg.len() {
            let seg_size =
                u16::from_be_bytes([original_jpeg[pos], original_jpeg[pos + 1]]) as usize;

            if pos + seg_size <= original_jpeg.len()
                && seg_size >= 8
                && original_jpeg[pos + 2..pos + 8] == EXIF_HEADER
            {
                pos += seg_size;
                continue;
            }
        }

        // Copy other segments as-is
        if marker == 0xD8 || marker == 0xD9 {
            // SOI / EOI -- no size field
            output.push(0xFF);
            output.push(marker);
        } else if (0xD0..=0xD7).contains(&marker) {
            // RST markers -- no size field
            output.push(0xFF);
            output.push(marker);
        } else if pos + 2 <= original_jpeg.len() {
            let seg_size =
                u16::from_be_bytes([original_jpeg[pos], original_jpeg[pos + 1]]) as usize;
            if pos + seg_size <= original_jpeg.len() {
                output.push(0xFF);
                output.push(marker);
                output.extend_from_slice(&original_jpeg[pos..pos + seg_size]);
                pos += seg_size;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if pos < original_jpeg.len() {
        output.extend_from_slice(&original_jpeg[pos..]);
    }

    Ok(output)
}

// ── Public API ────────────────────────────────────────────────────────────────

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

    let (tags_to_replace, new_fields) = build_exif_changes(req)?;
    let original_data = std::fs::read(file_path)?;
    let exif_bytes = build_merged_exif_bytes(&original_data, &tags_to_replace, &new_fields)?;
    let new_jpeg = build_jpeg_with_exif(&original_data, &exif_bytes)?;
    std::fs::write(file_path, &new_jpeg)?;

    Ok(())
}

/// Write multiple EXIF fields to a RAF (Fujifilm RAW) file.
///
/// RAF files contain an embedded JPEG with EXIF data. This function:
/// 1. Extracts the embedded JPEG from the RAF container
/// 2. Modifies its EXIF using the same merge strategy as JPEG
/// 3. Reconstructs the RAF file with the updated JPEG and adjusted header offsets
pub fn write_exif_fields_raf(
    file_path: &Path,
    req: &ExifWriteRequest,
) -> Result<(), crate::error::AppError> {
    if !file_path.exists() {
        return Err(crate::error::AppError::General(format!(
            "File not found: {}",
            file_path.display()
        )));
    }

    let raf_data = std::fs::read(file_path)?;

    // Validate RAF magic
    if raf_data.len() < 108 {
        return Err(crate::error::AppError::General(
            "File too small to be a valid RAF file".to_string(),
        ));
    }
    if !raf_data[0..16].starts_with(b"FUJIFILMCCD-RAW") {
        return Err(crate::error::AppError::General(
            "Not a valid RAF file: missing FUJIFILMCCD-RAW magic".to_string(),
        ));
    }

    // Parse header offsets (Big Endian i32 at fixed positions)
    let jpg_offset = BigEndian::read_i32(&raf_data[84..88]) as usize;
    let jpg_length = BigEndian::read_i32(&raf_data[88..92]) as usize;
    let cfa_header_offset = BigEndian::read_i32(&raf_data[92..96]);
    let cfa_offset = BigEndian::read_i32(&raf_data[100..104]);

    // Validate JPEG region is within file bounds
    if jpg_offset + jpg_length > raf_data.len() {
        return Err(crate::error::AppError::General(
            "RAF header indicates JPEG region beyond file bounds".to_string(),
        ));
    }
    let embedded_jpeg = &raf_data[jpg_offset..jpg_offset + jpg_length];
    if embedded_jpeg.len() < 2 || embedded_jpeg[0] != 0xFF || embedded_jpeg[1] != 0xD8 {
        return Err(crate::error::AppError::General(
            "Embedded JPEG in RAF does not start with SOI marker".to_string(),
        ));
    }

    // Modify EXIF in the embedded JPEG using shared helpers
    let (tags_to_replace, new_fields) = build_exif_changes(req)?;
    let exif_bytes = build_merged_exif_bytes(embedded_jpeg, &tags_to_replace, &new_fields)?;
    let new_jpeg = build_jpeg_with_exif(embedded_jpeg, &exif_bytes)?;

    // Reconstruct the RAF file
    let size_diff = new_jpeg.len() as i64 - jpg_length as i64;

    let mut output = Vec::with_capacity((raf_data.len() as i64 + size_diff) as usize);
    // Everything before the JPEG (header + gap)
    output.extend_from_slice(&raf_data[..jpg_offset]);
    // New JPEG data
    output.extend_from_slice(&new_jpeg);
    // Everything after the old JPEG (CFA header + CFA data + trailing)
    output.extend_from_slice(&raf_data[jpg_offset + jpg_length..]);

    // Patch header: update jpg_length
    BigEndian::write_i32(&mut output[88..92], new_jpeg.len() as i32);

    // Adjust CFA offsets if they come after the JPEG section
    if size_diff != 0 {
        if cfa_header_offset as usize > jpg_offset {
            let new_val = cfa_header_offset as i64 + size_diff;
            if new_val < 0 {
                return Err(crate::error::AppError::General(
                    "EXIF modification would corrupt RAF structure: CFA header offset underflow"
                        .to_string(),
                ));
            }
            BigEndian::write_i32(&mut output[92..96], new_val as i32);
        }
        if cfa_offset as usize > jpg_offset {
            let new_val = cfa_offset as i64 + size_diff;
            if new_val < 0 {
                return Err(crate::error::AppError::General(
                    "EXIF modification would corrupt RAF structure: CFA data offset underflow"
                        .to_string(),
                ));
            }
            BigEndian::write_i32(&mut output[100..104], new_val as i32);
        }
    }

    std::fs::write(file_path, &output)?;

    Ok(())
}

// ── Utility functions ─────────────────────────────────────────────────────────

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::exif::ExifWriteRequest;
    use crate::services::exif_service::read_exif_service;
    use std::fs;
    use std::path::PathBuf;

    /// Root directory of the project (src-tauri/)
    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// Copy a test data file to the test_output directory and return the destination path.
    /// Uses a unique suffix to avoid collisions between test cases.
    fn copy_test_file(filename: &str, suffix: &str) -> PathBuf {
        let src = project_root().join("test_data").join(filename);
        let dest_dir = project_root().join("test_output");
        fs::create_dir_all(&dest_dir).unwrap();

        let stem = filename.strip_suffix(".jpg")
            .or_else(|| filename.strip_suffix(".RAF"))
            .unwrap_or("sample");
        let ext = if filename.ends_with(".RAF") { ".RAF" } else { ".jpg" };
        let dest_name = format!("{}_{}{}", stem, suffix, ext);
        let dest = dest_dir.join(&dest_name);
        fs::copy(&src, &dest).unwrap();
        dest
    }

    // ── Helper: write + read-back verification ────────────────────────────────

    /// Write EXIF fields to a JPEG file and read them back.
    fn write_and_readback(path: &Path, req: &ExifWriteRequest) -> crate::models::exif::ExifInfo {
        write_exif_fields_jpg(path, req).expect("write_exif_fields_jpg failed");
        read_exif_service::read_exif_jpg(path).expect("read_exif_jpg failed")
    }

    /// Write EXIF fields to a RAF file and read them back.
    fn write_and_readback_raf(path: &Path, req: &ExifWriteRequest) -> crate::models::exif::ExifInfo {
        write_exif_fields_raf(path, req).expect("write_exif_fields_raf failed");
        read_exif_service::read_exif_raf(path).expect("read_exif_raf failed")
    }

    // ══════════════════════════════════════════════════════════════════════════
    // ── JPG tests ─────────────────────────────────────────────────────────────
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_write_and_read_datetime_jpg() {
        let path = copy_test_file("sample.jpg", "datetime");
        let req = ExifWriteRequest {
            datetime: Some("2025:01:15 10:30:00".to_string()),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert!(info.datetime.as_ref().map_or(false, |dt| dt.contains("2025") && dt.contains("01") && dt.contains("15")),
            "DateTime mismatch: {:?}", info.datetime);
    }

    #[test]
    fn test_write_and_read_camera_model_jpg() {
        let path = copy_test_file("sample.jpg", "cam_model");
        let req = ExifWriteRequest {
            camera_model: Some("TestCamera X100".to_string()),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert_eq!(info.camera_model.as_deref(), Some("TestCamera X100"));
    }

    #[test]
    fn test_write_and_read_gps_jpg() {
        let path = copy_test_file("sample.jpg", "gps");
        let req = ExifWriteRequest {
            gps_latitude: Some(35.681236),
            gps_longitude: Some(139.767125),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert!(info.gps_latitude.is_some(), "GPS latitude should be set");
        assert!(info.gps_longitude.is_some(), "GPS longitude should be set");
        let lat = info.gps_latitude.unwrap();
        let lon = info.gps_longitude.unwrap();
        assert!((lat - 35.681236).abs() < 0.001, "Latitude mismatch: {}", lat);
        assert!((lon - 139.767125).abs() < 0.001, "Longitude mismatch: {}", lon);
    }

    #[test]
    fn test_write_and_read_rating_jpg() {
        let path = copy_test_file("sample.jpg", "rating");
        let req = ExifWriteRequest {
            rating: Some(4),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert_eq!(info.rating, Some(4), "Rating should be 4");
    }

    #[test]
    fn test_write_and_read_focal_length_jpg() {
        let path = copy_test_file("sample.jpg", "focal");
        let req = ExifWriteRequest {
            focal_length: Some(50.0),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert!(info.focal_length.is_some(), "Focal length should be set");
        let fl = info.focal_length.unwrap();
        assert!((fl - 50.0).abs() < 1.0, "Focal length mismatch: {}", fl);
    }

    #[test]
    fn test_write_and_read_shutter_speed_jpg() {
        let path = copy_test_file("sample.jpg", "shutter");
        let req = ExifWriteRequest {
            shutter_speed: Some("1/250".to_string()),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert!(info.shutter_speed.is_some(), "Shutter speed should be set");
    }

    #[test]
    fn test_write_and_read_aperture_jpg() {
        let path = copy_test_file("sample.jpg", "aperture");
        let req = ExifWriteRequest {
            aperture: Some(2.8),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert!(info.aperture.is_some(), "Aperture should be set");
        let ap = info.aperture.unwrap();
        assert!((ap - 2.8).abs() < 0.1, "Aperture mismatch: {}", ap);
    }

    #[test]
    fn test_write_and_read_iso_jpg() {
        let path = copy_test_file("sample.jpg", "iso");
        let req = ExifWriteRequest {
            iso: Some(800),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert_eq!(info.iso, Some(800), "ISO should be 800");
    }

    // ── Multi-field and overwrite tests ───────────────────────────────────────

    #[test]
    fn test_write_multiple_fields_jpg() {
        let path = copy_test_file("sample.jpg", "multi");
        let req = ExifWriteRequest {
            datetime: Some("2025:06:01 12:00:00".to_string()),
            camera_model: Some("MultiTest Camera".to_string()),
            focal_length: Some(35.0),
            aperture: Some(4.0),
            iso: Some(200),
            rating: Some(5),
            ..Default::default()
        };
        let info = write_and_readback(&path, &req);
        assert!(info.datetime.as_ref().map_or(false, |dt| dt.contains("2025") && dt.contains("06") && dt.contains("01")),
            "DateTime mismatch: {:?}", info.datetime);
        assert_eq!(info.camera_model.as_deref(), Some("MultiTest Camera"));
        assert!(info.focal_length.is_some());
        assert!(info.aperture.is_some());
        assert_eq!(info.iso, Some(200));
        assert_eq!(info.rating, Some(5));
    }

    #[test]
    fn test_overwrite_existing_field_jpg() {
        let path = copy_test_file("sample.jpg", "overwrite");
        // First write
        let req1 = ExifWriteRequest {
            camera_model: Some("FirstModel".to_string()),
            ..Default::default()
        };
        let info1 = write_and_readback(&path, &req1);
        assert_eq!(info1.camera_model.as_deref(), Some("FirstModel"));
        // Overwrite
        let req2 = ExifWriteRequest {
            camera_model: Some("SecondModel".to_string()),
            ..Default::default()
        };
        let info2 = write_and_readback(&path, &req2);
        assert_eq!(info2.camera_model.as_deref(), Some("SecondModel"));
    }

    // ── Batch parallel write test ─────────────────────────────────────────────

    #[test]
    fn test_batch_write_parallel_jpg() {
        // Prepare multiple test file copies
        let paths: Vec<String> = (0..4)
            .map(|i| {
                let p = copy_test_file("sample.jpg", &format!("batch_{}", i));
                p.to_string_lossy().to_string()
            })
            .collect();

        let req = ExifWriteRequest {
            camera_model: Some("BatchTest".to_string()),
            rating: Some(3),
            ..Default::default()
        };

        crate::services::exif_service::write_exif_fields(&paths, &req)
            .expect("batch write_exif_fields failed");

        // Verify each file
        for path_str in &paths {
            let info = read_exif_service::read_exif_jpg(Path::new(path_str))
                .expect("read_exif_jpg failed");
            assert_eq!(info.camera_model.as_deref(), Some("BatchTest"));
            assert_eq!(info.rating, Some(3));
        }
    }

    // ── RAF read test ────────────────────────────────────────────────────────

    #[test]
    fn test_read_exif_raf() {
        let path = project_root().join("test_data").join("sample.RAF");
        let info = read_exif_service::read_exif_raf(&path)
            .expect("read_exif_raf failed");
        // RAF should have some EXIF data
        assert!(info.camera_model.is_some() || info.datetime.is_some(),
            "RAF file should contain at least camera model or datetime");
    }

    // ── Rating validation test ───────────────────────────────────────────────

    #[test]
    fn test_rating_validation() {
        let path = copy_test_file("sample.jpg", "rating_invalid");
        let req = ExifWriteRequest {
            rating: Some(6), // invalid: > 5
            ..Default::default()
        };
        let result = write_exif_fields_jpg(&path, &req);
        assert!(result.is_err(), "Rating > 5 should be rejected");
    }

    // ── Batch GPS via write_exif_fields ───────────────────────────────────────

    #[test]
    fn test_batch_write_gps_via_write_exif_fields() {
        let paths: Vec<String> = (0..3)
            .map(|i| {
                let p = copy_test_file("sample.jpg", &format!("batch_gps_{}", i));
                p.to_string_lossy().to_string()
            })
            .collect();

        let req = ExifWriteRequest {
            gps_latitude: Some(40.7128),
            gps_longitude: Some(-74.0060),
            ..Default::default()
        };

        crate::services::exif_service::write_exif_fields(&paths, &req)
            .expect("batch GPS write failed");

        for path_str in &paths {
            let info = read_exif_service::read_exif_jpg(Path::new(path_str))
                .expect("read_exif_jpg failed");
            assert!(info.gps_latitude.is_some(), "GPS latitude should be set");
            assert!(info.gps_longitude.is_some(), "GPS longitude should be set");
            let lat = info.gps_latitude.unwrap();
            let lon = info.gps_longitude.unwrap();
            assert!((lat - 40.7128).abs() < 0.001, "Latitude mismatch: {}", lat);
            assert!((lon - (-74.0060)).abs() < 0.001, "Longitude mismatch: {}", lon);
        }
    }

    // ── Shutter speed parsing tests ───────────────────────────────────────────

    #[test]
    fn test_parse_shutter_speed_fraction() {
        assert_eq!(parse_shutter_speed("1/500"), Some((1, 500)));
        assert_eq!(parse_shutter_speed("1/ 500"), Some((1, 500)));
    }

    #[test]
    fn test_parse_shutter_speed_whole() {
        assert_eq!(parse_shutter_speed("2"), Some((2, 1)));
        assert_eq!(parse_shutter_speed("2.0"), Some((2, 1)));
    }

    #[test]
    fn test_parse_shutter_speed_decimal() {
        assert_eq!(parse_shutter_speed("0.5"), Some((1, 2)));
    }

    #[test]
    fn test_parse_shutter_speed_invalid() {
        assert_eq!(parse_shutter_speed("abc"), None);
        assert_eq!(parse_shutter_speed("1/0"), None);
        assert_eq!(parse_shutter_speed("-1"), None);
    }

    // ── DMS conversion test ───────────────────────────────────────────────────

    #[test]
    fn test_decimal_degrees_to_dms() {
        let dms = decimal_degrees_to_dms(40.7128);
        assert_eq!(dms[0].num, 40);  // degrees
        assert_eq!(dms[1].num, 42);  // minutes
        // seconds: 0.768 * 60 = 46.08 → 46080000 / 1000000
        assert!((dms[2].to_f64() - 46.08).abs() < 0.1, "Seconds mismatch: {}", dms[2].to_f64());
    }

    // ══════════════════════════════════════════════════════════════════════════
    // ── RAF write tests ───────────────────────────────────────────────────────
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_write_and_read_datetime_raf() {
        let path = copy_test_file("sample.RAF", "datetime");
        let req = ExifWriteRequest {
            datetime: Some("2025:01:15 10:30:00".to_string()),
            ..Default::default()
        };
        let info = write_and_readback_raf(&path, &req);
        assert!(
            info.datetime.as_ref().map_or(false, |dt| dt.contains("2025") && dt.contains("01") && dt.contains("15")),
            "DateTime mismatch: {:?}", info.datetime
        );
    }

    #[test]
    fn test_write_and_read_camera_model_raf() {
        let path = copy_test_file("sample.RAF", "cam_model");
        let req = ExifWriteRequest {
            camera_model: Some("TestCamera X100".to_string()),
            ..Default::default()
        };
        let info = write_and_readback_raf(&path, &req);
        assert_eq!(info.camera_model.as_deref(), Some("TestCamera X100"));
    }

    #[test]
    fn test_write_and_read_gps_raf() {
        let path = copy_test_file("sample.RAF", "gps");
        let req = ExifWriteRequest {
            gps_latitude: Some(35.681236),
            gps_longitude: Some(139.767125),
            ..Default::default()
        };
        let info = write_and_readback_raf(&path, &req);
        assert!(info.gps_latitude.is_some(), "GPS latitude should be set");
        assert!(info.gps_longitude.is_some(), "GPS longitude should be set");
        let lat = info.gps_latitude.unwrap();
        let lon = info.gps_longitude.unwrap();
        assert!((lat - 35.681236).abs() < 0.001, "Latitude mismatch: {}", lat);
        assert!((lon - 139.767125).abs() < 0.001, "Longitude mismatch: {}", lon);
    }

    #[test]
    fn test_write_and_read_rating_raf() {
        let path = copy_test_file("sample.RAF", "rating");
        let req = ExifWriteRequest {
            rating: Some(4),
            ..Default::default()
        };
        let info = write_and_readback_raf(&path, &req);
        assert_eq!(info.rating, Some(4), "Rating should be 4");
    }

    #[test]
    fn test_write_and_read_iso_raf() {
        let path = copy_test_file("sample.RAF", "iso");
        let req = ExifWriteRequest {
            iso: Some(800),
            ..Default::default()
        };
        let info = write_and_readback_raf(&path, &req);
        assert_eq!(info.iso, Some(800), "ISO should be 800");
    }

    #[test]
    fn test_write_multiple_fields_raf() {
        let path = copy_test_file("sample.RAF", "multi");
        let req = ExifWriteRequest {
            datetime: Some("2025:06:01 12:00:00".to_string()),
            camera_model: Some("MultiTest RAF Camera".to_string()),
            focal_length: Some(35.0),
            aperture: Some(4.0),
            iso: Some(200),
            rating: Some(5),
            ..Default::default()
        };
        let info = write_and_readback_raf(&path, &req);
        assert!(
            info.datetime.as_ref().map_or(false, |dt| dt.contains("2025") && dt.contains("06") && dt.contains("01")),
            "DateTime mismatch: {:?}", info.datetime
        );
        assert_eq!(info.camera_model.as_deref(), Some("MultiTest RAF Camera"));
        assert!(info.focal_length.is_some());
        assert!(info.aperture.is_some());
        assert_eq!(info.iso, Some(200));
        assert_eq!(info.rating, Some(5));
    }

    #[test]
    fn test_overwrite_existing_field_raf() {
        let path = copy_test_file("sample.RAF", "overwrite");
        // First write
        let req1 = ExifWriteRequest {
            camera_model: Some("FirstModel".to_string()),
            ..Default::default()
        };
        let info1 = write_and_readback_raf(&path, &req1);
        assert_eq!(info1.camera_model.as_deref(), Some("FirstModel"));
        // Overwrite
        let req2 = ExifWriteRequest {
            camera_model: Some("SecondModel".to_string()),
            ..Default::default()
        };
        let info2 = write_and_readback_raf(&path, &req2);
        assert_eq!(info2.camera_model.as_deref(), Some("SecondModel"));
    }

    #[test]
    fn test_raf_structure_preserved() {
        use crate::services::raw_decoders::raf_decoder::RafDecoder;

        let path = copy_test_file("sample.RAF", "structure");
        let req = ExifWriteRequest {
            camera_model: Some("StructureTest".to_string()),
            rating: Some(3),
            ..Default::default()
        };
        write_exif_fields_raf(&path, &req).expect("write_exif_fields_raf failed");

        // Verify RAF magic is intact
        let data = fs::read(&path).unwrap();
        assert!(
            data[0..16].starts_with(b"FUJIFILMCCD-RAW"),
            "RAF magic should be preserved after write"
        );

        // Verify RafDecoder can still parse the modified file
        let decoder = RafDecoder::new(&path).expect("RafDecoder should parse modified RAF");
        assert!(decoder.get_jpeg_size() > 0, "JPEG data should still be present");
        assert!(decoder.get_cfa_record_count() > 0, "CFA records should still be present");
    }

    #[test]
    fn test_batch_write_mixed_jpg_raf() {
        let jpg_path = copy_test_file("sample.jpg", "mixed_ok");
        let raf_path = copy_test_file("sample.RAF", "mixed_ok");

        let paths = vec![
            jpg_path.to_string_lossy().to_string(),
            raf_path.to_string_lossy().to_string(),
        ];

        let req = ExifWriteRequest {
            rating: Some(3),
            ..Default::default()
        };

        crate::services::exif_service::write_exif_fields(&paths, &req)
            .expect("batch write with JPG+RAF should succeed");

        // Verify JPG
        let jpg_info = read_exif_service::read_exif_jpg(Path::new(&paths[0]))
            .expect("read_exif_jpg failed");
        assert_eq!(jpg_info.rating, Some(3));

        // Verify RAF
        let raf_info = read_exif_service::read_exif_raf(Path::new(&paths[1]))
            .expect("read_exif_raf failed");
        assert_eq!(raf_info.rating, Some(3));
    }

    #[test]
    fn test_rating_validation_raf() {
        let path = copy_test_file("sample.RAF", "rating_invalid");
        let original_size = fs::metadata(&path).unwrap().len();

        let req = ExifWriteRequest {
            rating: Some(6), // invalid: > 5
            ..Default::default()
        };
        let result = write_exif_fields_raf(&path, &req);
        assert!(result.is_err(), "Rating > 5 should be rejected");

        // Verify file was not modified
        let after_size = fs::metadata(&path).unwrap().len();
        assert_eq!(original_size, after_size, "RAF file should not be modified on validation error");
    }
}
