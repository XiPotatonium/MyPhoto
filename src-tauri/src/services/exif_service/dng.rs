use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::exif::{ExifInfo, ExifWriteRequest};
use super::common;
use super::tiff;

// ── Read ──────────────────────────────────────────────────────────────────────

/// DNG 预览 JPEG 前缀读取上限（512KB），足以覆盖预览头部的 APP1(EXIF)。
const DNG_PREFIX_LEN: usize = 512 * 1024;

/// Read EXIF info from an Adobe DNG raw file.
///
/// Strategy:
/// 1. 轻量：仅解析 IFD 链 + 读预览 JPEG 前缀（512KB）解析 EXIF，宽高从 tag 256/257 填充
/// 2. Fallback: 构造完整 DngDecoder，从整段预览 JPEG 解析 EXIF
/// 3. Fallback: 直接把 DNG（TIFF-based）作为容器读取 EXIF
pub fn read_exif_dng(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    use crate::services::raw_decoders::dng_decoder::DngDecoder;

    if let Some(info) = read_exif_dng_prefix(file_path) {
        return Ok(info);
    }

    let decoder = DngDecoder::new(file_path)
        .map_err(|e| crate::error::AppError::General(format!("Failed to decode DNG: {}", e)))?;

    if !decoder.preview.data.is_empty() {
        // Try reading EXIF from preview JPEG
        let mut cursor = std::io::Cursor::new(&decoder.preview.data);
        if let Ok(exif_data) = exif::Reader::new().read_from_container(&mut cursor) {
            return common::parse_exif_data(&exif_data);
        }
    }

    // Fallback: read EXIF directly from DNG file (TIFF-based format)
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(&file);
    let exif_data = exif::Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| crate::error::AppError::Exif(e.to_string()))?;
    common::parse_exif_data(&exif_data)
}

/// 轻量前缀路径：仅解析 IFD 链，从主 IFD 及子 IFD 依次尝试读取预览 JPEG 前缀解析 EXIF。
/// 任一步失败返回 None，交由调用方回退整段路径。
fn read_exif_dng_prefix(file_path: &Path) -> Option<ExifInfo> {
    use crate::services::raw_decoders::dng_decoder::{DngDecoder, DngIfdChain, DngPreviewData};

    let DngIfdChain {
        header,
        main_ifd,
        sub_ifds,
    } = DngDecoder::parse_ifds(file_path).ok()?;

    // 主 IFD 优先（与原整段路径一致），再尝试各子 IFD
    for ifd in std::iter::once(&main_ifd).chain(sub_ifds.iter()) {
        let preview = match DngPreviewData::extract_prefix(
            file_path,
            ifd,
            header.byte_order,
            DNG_PREFIX_LEN,
        ) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if preview.data.is_empty() {
            continue;
        }
        let mut cursor = std::io::Cursor::new(&preview.data);
        if let Ok(exif_data) = exif::Reader::new().read_from_container(&mut cursor) {
            if let Ok(mut info) = common::parse_exif_data(&exif_data) {
                // 宽高从 IFD 的 tag 256/257 补全，避免 mod.rs 二次读取
                if (info.image_width.is_none() || info.image_height.is_none())
                    && preview.width > 0
                    && preview.height > 0
                {
                    info.image_width = Some(preview.width);
                    info.image_height = Some(preview.height);
                }
                return Some(info);
            }
        }
    }

    None
}

// ── Write ─────────────────────────────────────────────────────────────────────

/// Write EXIF fields to a DNG file.
///
/// DNG is based on TIFF format, so we reuse the TIFF IFD write logic directly.
pub fn write_exif_fields_dng(
    file_path: &Path,
    req: &ExifWriteRequest,
) -> Result<(), crate::error::AppError> {
    // DNG is TIFF-based, use the same IFD manipulation strategy
    tiff::write_exif_fields_tiff(file_path, req)
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

        let ext_pos = filename.rfind('.').unwrap_or(filename.len());
        let stem = &filename[..ext_pos];
        let ext = &filename[ext_pos..];
        let dest_name = format!("{}_{}{}", stem, suffix, ext);
        let dest = dest_dir.join(&dest_name);
        fs::copy(&src, &dest).unwrap();
        dest
    }

    fn write_and_readback_dng(path: &Path, req: &ExifWriteRequest) -> ExifInfo {
        write_exif_fields_dng(path, req).expect("write_exif_fields_dng failed");
        read_exif_dng(path).expect("read_exif_dng failed")
    }

    #[test]
    fn test_read_exif_dng() {
        let path = project_root().join("test_data").join("DSCF0409.dng");
        let result = read_exif_dng(&path);
        assert!(result.is_ok(), "Should be able to read DNG file: {:?}", result.err());
        let info = result.unwrap();
        // DNG files typically have camera info
        assert!(
            info.camera_model.is_some() || info.camera_make.is_some() || info.datetime.is_some(),
            "DNG file should contain some EXIF data"
        );
    }

    #[test]
    fn test_write_and_read_rating_dng() {
        let path = copy_test_file("DSCF0409.dng", "rating");
        let req = ExifWriteRequest {
            rating: Some(3),
            ..Default::default()
        };
        let info = write_and_readback_dng(&path, &req);
        assert_eq!(info.rating, Some(3), "Rating should be 3");
    }

    #[test]
    fn test_write_and_read_datetime_dng() {
        let path = copy_test_file("DSCF0409.dng", "datetime");
        let req = ExifWriteRequest {
            datetime: Some("2025:04:10 16:45:00".to_string()),
            ..Default::default()
        };
        let info = write_and_readback_dng(&path, &req);
        assert!(
            info.datetime.as_ref().map_or(false, |dt| dt.contains("2025") && dt.contains("04") && dt.contains("10")),
            "DateTime mismatch: {:?}", info.datetime
        );
    }

    #[test]
    fn test_write_and_read_camera_model_dng() {
        let path = copy_test_file("DSCF0409.dng", "cam_model");
        let req = ExifWriteRequest {
            camera_model: Some("DNG Test Camera".to_string()),
            ..Default::default()
        };
        let info = write_and_readback_dng(&path, &req);
        assert_eq!(info.camera_model.as_deref(), Some("DNG Test Camera"));
    }

    #[test]
    fn test_write_and_read_iso_dng() {
        let path = copy_test_file("DSCF0409.dng", "iso");
        let req = ExifWriteRequest {
            iso: Some(1600),
            ..Default::default()
        };
        let info = write_and_readback_dng(&path, &req);
        assert_eq!(info.iso, Some(1600), "ISO should be 1600");
    }

    #[test]
    fn test_write_and_read_gps_dng() {
        let path = copy_test_file("DSCF0409.dng", "gps");
        let req = ExifWriteRequest {
            gps_latitude: Some(48.8566),
            gps_longitude: Some(2.3522),
            ..Default::default()
        };
        let info = write_and_readback_dng(&path, &req);
        assert!(info.gps_latitude.is_some(), "GPS latitude should be set");
        assert!(info.gps_longitude.is_some(), "GPS longitude should be set");
        let lat = info.gps_latitude.unwrap();
        let lon = info.gps_longitude.unwrap();
        assert!((lat - 48.8566).abs() < 0.001, "Latitude mismatch: {}", lat);
        assert!((lon - 2.3522).abs() < 0.001, "Longitude mismatch: {}", lon);
    }

    #[test]
    fn test_write_multiple_fields_dng() {
        let path = copy_test_file("DSCF0409.dng", "multi");
        let req = ExifWriteRequest {
            datetime: Some("2025:08:15 09:00:00".to_string()),
            camera_model: Some("MultiTest DNG".to_string()),
            iso: Some(3200),
            rating: Some(5),
            ..Default::default()
        };
        let info = write_and_readback_dng(&path, &req);
        assert!(info.datetime.as_ref().map_or(false, |dt| dt.contains("2025")));
        assert_eq!(info.camera_model.as_deref(), Some("MultiTest DNG"));
        assert_eq!(info.iso, Some(3200));
        assert_eq!(info.rating, Some(5));
    }

    #[test]
    fn test_rating_validation_dng() {
        let path = copy_test_file("DSCF0409.dng", "rating_invalid");
        let req = ExifWriteRequest {
            rating: Some(6),
            ..Default::default()
        };
        let result = write_exif_fields_dng(&path, &req);
        assert!(result.is_err(), "Rating > 5 should be rejected");
    }
}
