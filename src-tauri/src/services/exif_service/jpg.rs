use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::exif::{ExifInfo, ExifWriteRequest};
use super::common;

// ── Read ──────────────────────────────────────────────────────────────────────

/// Read EXIF info from a JPEG/JFIF file.
pub fn read_exif_jpg(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(&file);
    let exif_data = exif::Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| crate::error::AppError::Exif(e.to_string()))?;
    common::parse_exif_data(&exif_data)
}

// ── Write ─────────────────────────────────────────────────────────────────────

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

    let (tags_to_replace, new_fields) = common::build_exif_changes(req)?;
    let original_data = std::fs::read(file_path)?;
    let exif_bytes = common::build_merged_exif_bytes(&original_data, &tags_to_replace, &new_fields)?;
    let new_jpeg = common::build_jpeg_with_exif(&original_data, &exif_bytes)?;
    std::fs::write(file_path, &new_jpeg)?;

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

    fn write_and_readback(path: &Path, req: &ExifWriteRequest) -> ExifInfo {
        write_exif_fields_jpg(path, req).expect("write_exif_fields_jpg failed");
        read_exif_jpg(path).expect("read_exif_jpg failed")
    }

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
        let req1 = ExifWriteRequest {
            camera_model: Some("FirstModel".to_string()),
            ..Default::default()
        };
        let info1 = write_and_readback(&path, &req1);
        assert_eq!(info1.camera_model.as_deref(), Some("FirstModel"));
        let req2 = ExifWriteRequest {
            camera_model: Some("SecondModel".to_string()),
            ..Default::default()
        };
        let info2 = write_and_readback(&path, &req2);
        assert_eq!(info2.camera_model.as_deref(), Some("SecondModel"));
    }

    #[test]
    fn test_batch_write_parallel_jpg() {
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

        for path_str in &paths {
            let info = read_exif_jpg(Path::new(path_str))
                .expect("read_exif_jpg failed");
            assert_eq!(info.camera_model.as_deref(), Some("BatchTest"));
            assert_eq!(info.rating, Some(3));
        }
    }

    #[test]
    fn test_rating_validation() {
        let path = copy_test_file("sample.jpg", "rating_invalid");
        let req = ExifWriteRequest {
            rating: Some(6),
            ..Default::default()
        };
        let result = write_exif_fields_jpg(&path, &req);
        assert!(result.is_err(), "Rating > 5 should be rejected");
    }

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
            let info = read_exif_jpg(Path::new(path_str))
                .expect("read_exif_jpg failed");
            assert!(info.gps_latitude.is_some(), "GPS latitude should be set");
            assert!(info.gps_longitude.is_some(), "GPS longitude should be set");
            let lat = info.gps_latitude.unwrap();
            let lon = info.gps_longitude.unwrap();
            assert!((lat - 40.7128).abs() < 0.001, "Latitude mismatch: {}", lat);
            assert!((lon - (-74.0060)).abs() < 0.001, "Longitude mismatch: {}", lon);
        }
    }

    #[test]
    fn test_parse_shutter_speed_fraction() {
        assert_eq!(common::parse_shutter_speed("1/500"), Some((1, 500)));
        assert_eq!(common::parse_shutter_speed("1/ 500"), Some((1, 500)));
    }

    #[test]
    fn test_parse_shutter_speed_whole() {
        assert_eq!(common::parse_shutter_speed("2"), Some((2, 1)));
        assert_eq!(common::parse_shutter_speed("2.0"), Some((2, 1)));
    }

    #[test]
    fn test_parse_shutter_speed_decimal() {
        assert_eq!(common::parse_shutter_speed("0.5"), Some((1, 2)));
    }

    #[test]
    fn test_parse_shutter_speed_invalid() {
        assert_eq!(common::parse_shutter_speed("abc"), None);
        assert_eq!(common::parse_shutter_speed("1/0"), None);
        assert_eq!(common::parse_shutter_speed("-1"), None);
    }

    #[test]
    fn test_decimal_degrees_to_dms() {
        let dms = common::decimal_degrees_to_dms(40.7128);
        assert_eq!(dms[0].num, 40);
        assert_eq!(dms[1].num, 42);
        assert!((dms[2].to_f64() - 46.08).abs() < 0.1, "Seconds mismatch: {}", dms[2].to_f64());
    }
}
