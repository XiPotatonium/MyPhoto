use std::path::Path;

use byteorder::{BigEndian, ByteOrder};

use crate::models::exif::{ExifInfo, ExifWriteRequest};
use super::common;

// ── Read ──────────────────────────────────────────────────────────────────────

/// Read EXIF info from a Fujifilm RAF raw file.
pub fn read_exif_raf(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    use crate::services::raw_decoders::raf_decoder::RafDecoder;

    let decoder = RafDecoder::new(file_path)
        .map_err(|e| crate::error::AppError::General(format!("Failed to decode RAF: {}", e)))?;

    let mut cursor = std::io::Cursor::new(&decoder.jpeg.data);
    let exif_data = exif::Reader::new()
        .read_from_container(&mut cursor)
        .map_err(|e| crate::error::AppError::Exif(e.to_string()))?;
    common::parse_exif_data(&exif_data)
}

// ── Write ─────────────────────────────────────────────────────────────────────

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
    let (tags_to_replace, new_fields) = common::build_exif_changes(req)?;
    let exif_bytes = common::build_merged_exif_bytes(embedded_jpeg, &tags_to_replace, &new_fields)?;
    let new_jpeg = common::build_jpeg_with_exif(embedded_jpeg, &exif_bytes)?;

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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::exif::ExifWriteRequest;
    use std::fs;
    use std::path::PathBuf;

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

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

    fn write_and_readback_raf(path: &Path, req: &ExifWriteRequest) -> ExifInfo {
        write_exif_fields_raf(path, req).expect("write_exif_fields_raf failed");
        read_exif_raf(path).expect("read_exif_raf failed")
    }

    #[test]
    fn test_read_exif_raf() {
        let path = project_root().join("test_data").join("sample.RAF");
        let info = read_exif_raf(&path).expect("read_exif_raf failed");
        assert!(info.camera_model.is_some() || info.datetime.is_some(),
            "RAF file should contain at least camera model or datetime");
    }

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
        let req1 = ExifWriteRequest {
            camera_model: Some("FirstModel".to_string()),
            ..Default::default()
        };
        let info1 = write_and_readback_raf(&path, &req1);
        assert_eq!(info1.camera_model.as_deref(), Some("FirstModel"));
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

        let data = fs::read(&path).unwrap();
        assert!(
            data[0..16].starts_with(b"FUJIFILMCCD-RAW"),
            "RAF magic should be preserved after write"
        );

        let decoder = RafDecoder::new(&path).expect("RafDecoder should parse modified RAF");
        assert!(decoder.get_jpeg_size() > 0, "JPEG data should still be present");
        assert!(decoder.get_cfa_record_count() > 0, "CFA records should still be present");
    }

    #[test]
    fn test_batch_write_mixed_jpg_raf() {
        use super::super::jpg;

        let jpg_src = project_root().join("test_data").join("sample.jpg");
        let raf_src = project_root().join("test_data").join("sample.RAF");
        let dest_dir = project_root().join("test_output");
        fs::create_dir_all(&dest_dir).unwrap();

        let jpg_dest = dest_dir.join("sample_mixed_ok.jpg");
        let raf_dest = dest_dir.join("sample_mixed_ok.RAF");
        fs::copy(&jpg_src, &jpg_dest).unwrap();
        fs::copy(&raf_src, &raf_dest).unwrap();

        let paths = vec![
            jpg_dest.to_string_lossy().to_string(),
            raf_dest.to_string_lossy().to_string(),
        ];

        let req = ExifWriteRequest {
            rating: Some(3),
            ..Default::default()
        };

        crate::services::exif_service::write_exif_fields(&paths, &req)
            .expect("batch write with JPG+RAF should succeed");

        let jpg_info = jpg::read_exif_jpg(Path::new(&paths[0]))
            .expect("read_exif_jpg failed");
        assert_eq!(jpg_info.rating, Some(3));

        let raf_info = read_exif_raf(Path::new(&paths[1]))
            .expect("read_exif_raf failed");
        assert_eq!(raf_info.rating, Some(3));
    }

    #[test]
    fn test_rating_validation_raf() {
        let path = copy_test_file("sample.RAF", "rating_invalid");
        let original_size = fs::metadata(&path).unwrap().len();

        let req = ExifWriteRequest {
            rating: Some(6),
            ..Default::default()
        };
        let result = write_exif_fields_raf(&path, &req);
        assert!(result.is_err(), "Rating > 5 should be rejected");

        let after_size = fs::metadata(&path).unwrap().len();
        assert_eq!(original_size, after_size, "RAF file should not be modified on validation error");
    }
}
