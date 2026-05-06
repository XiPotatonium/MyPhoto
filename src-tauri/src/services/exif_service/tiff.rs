use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::exif::{ExifInfo, ExifWriteRequest};
use super::common::{self, TiffByteOrder, TiffIfdEntry};

// ── TIFF IFD Tag Constants ────────────────────────────────────────────────────

const TAG_DATETIME: u16 = 306;
const TAG_MODEL: u16 = 272;
const TAG_EXIF_IFD_POINTER: u16 = 34665;
const TAG_GPS_IFD_POINTER: u16 = 34853;
const TAG_RATING: u16 = 0x4746;
const TAG_RATING_PERCENT: u16 = 0x4749;

// Exif sub-IFD tags
const TAG_DATETIME_ORIGINAL: u16 = 36867;
const TAG_EXPOSURE_TIME: u16 = 33434;
const TAG_FNUMBER: u16 = 33437;
const TAG_ISO: u16 = 34855;
const TAG_FOCAL_LENGTH: u16 = 37386;
const TAG_LENS_MODEL: u16 = 42036;

// GPS IFD tags
const TAG_GPS_VERSION_ID: u16 = 0;
const TAG_GPS_LATITUDE_REF: u16 = 1;
const TAG_GPS_LATITUDE: u16 = 2;
const TAG_GPS_LONGITUDE_REF: u16 = 3;
const TAG_GPS_LONGITUDE: u16 = 4;

// TIFF field types
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_RATIONAL: u16 = 5;
const TYPE_BYTE: u16 = 1;

// ── Read ──────────────────────────────────────────────────────────────────────

/// Read EXIF info from a TIFF file.
pub fn read_exif_tiff(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(&file);
    let exif_data = exif::Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| crate::error::AppError::Exif(e.to_string()))?;
    common::parse_exif_data(&exif_data)
}

// ── Write ─────────────────────────────────────────────────────────────────────

/// Write EXIF fields to a TIFF file using direct IFD manipulation.
///
/// Strategy:
/// 1. Parse TIFF header and IFD0
/// 2. Locate/create Exif sub-IFD and GPS IFD
/// 3. Modify/add entries, appending new data at end of file
/// 4. Rebuild affected IFDs at end of file, update pointers
pub fn write_exif_fields_tiff(
    file_path: &Path,
    req: &ExifWriteRequest,
) -> Result<(), crate::error::AppError> {
    if !file_path.exists() {
        return Err(crate::error::AppError::General(format!(
            "File not found: {}",
            file_path.display()
        )));
    }

    let mut data = std::fs::read(file_path)?;
    let (bo, ifd0_offset) = common::parse_tiff_header(&data)?;

    // Parse IFD0
    let (mut ifd0_entries, ifd0_next) = common::parse_ifd(&data, ifd0_offset, bo)?;

    // Find or note Exif sub-IFD and GPS IFD pointers
    let exif_ifd_offset = ifd0_entries.iter()
        .find(|e| e.tag == TAG_EXIF_IFD_POINTER)
        .map(|e| e.value_offset);
    let gps_ifd_offset = ifd0_entries.iter()
        .find(|e| e.tag == TAG_GPS_IFD_POINTER)
        .map(|e| e.value_offset);

    // Parse existing sub-IFDs
    let mut exif_ifd_entries = if let Some(offset) = exif_ifd_offset {
        common::parse_ifd(&data, offset, bo).map(|(e, _)| e).unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut gps_ifd_entries = if let Some(offset) = gps_ifd_offset {
        common::parse_ifd(&data, offset, bo).map(|(e, _)| e).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Track whether we need to rebuild sub-IFDs
    let mut ifd0_modified = false;
    let mut exif_ifd_modified = false;
    let mut gps_ifd_modified = false;

    // ── Apply changes to IFD0 ─────────────────────────────────────────────────

    if let Some(ref dt) = req.datetime {
        let dt_bytes = format!("{}\0", dt);
        let offset = append_data(&mut data, dt_bytes.as_bytes());
        set_or_add_entry(&mut ifd0_entries, TAG_DATETIME, TYPE_ASCII, dt_bytes.len() as u32, offset, bo);
        ifd0_modified = true;
    }

    if let Some(ref model) = req.camera_model {
        let model_bytes = format!("{}\0", model);
        let offset = append_data(&mut data, model_bytes.as_bytes());
        set_or_add_entry(&mut ifd0_entries, TAG_MODEL, TYPE_ASCII, model_bytes.len() as u32, offset, bo);
        ifd0_modified = true;
    }

    if let Some(rating) = req.rating {
        if rating > 5 {
            return Err(crate::error::AppError::General(
                "Rating must be between 0 and 5".to_string(),
            ));
        }
        let val = encode_short_inline(rating as u16, bo);
        set_or_add_entry(&mut ifd0_entries, TAG_RATING, TYPE_SHORT, 1, val, bo);
        let percent_val = encode_short_inline((rating as u16) * 20, bo);
        set_or_add_entry(&mut ifd0_entries, TAG_RATING_PERCENT, TYPE_SHORT, 1, percent_val, bo);
        ifd0_modified = true;
    }

    // ── Apply changes to Exif sub-IFD ─────────────────────────────────────────

    if let Some(ref dt) = req.datetime {
        let dt_bytes = format!("{}\0", dt);
        let offset = append_data(&mut data, dt_bytes.as_bytes());
        set_or_add_entry(&mut exif_ifd_entries, TAG_DATETIME_ORIGINAL, TYPE_ASCII, dt_bytes.len() as u32, offset, bo);
        exif_ifd_modified = true;
    }

    if let Some(ref ss) = req.shutter_speed {
        let (num, denom) = common::parse_shutter_speed(ss).ok_or_else(|| {
            crate::error::AppError::General(format!(
                "Invalid shutter speed format: '{}'. Use '1/500' or '2'.", ss
            ))
        })?;
        let offset = append_rational(&mut data, num, denom, bo);
        set_or_add_entry(&mut exif_ifd_entries, TAG_EXPOSURE_TIME, TYPE_RATIONAL, 1, offset, bo);
        exif_ifd_modified = true;
    }

    if let Some(ap) = req.aperture {
        let num = (ap * 100.0).round() as u32;
        let offset = append_rational(&mut data, num, 100, bo);
        set_or_add_entry(&mut exif_ifd_entries, TAG_FNUMBER, TYPE_RATIONAL, 1, offset, bo);
        exif_ifd_modified = true;
    }

    if let Some(iso) = req.iso {
        let val = encode_short_inline(iso.min(65535) as u16, bo);
        set_or_add_entry(&mut exif_ifd_entries, TAG_ISO, TYPE_SHORT, 1, val, bo);
        exif_ifd_modified = true;
    }

    if let Some(fl) = req.focal_length {
        let num = (fl * 1000.0).round() as u32;
        let offset = append_rational(&mut data, num, 1000, bo);
        set_or_add_entry(&mut exif_ifd_entries, TAG_FOCAL_LENGTH, TYPE_RATIONAL, 1, offset, bo);
        exif_ifd_modified = true;
    }

    if let Some(ref lens) = req.lens_model {
        let lens_bytes = format!("{}\0", lens);
        let offset = append_data(&mut data, lens_bytes.as_bytes());
        set_or_add_entry(&mut exif_ifd_entries, TAG_LENS_MODEL, TYPE_ASCII, lens_bytes.len() as u32, offset, bo);
        exif_ifd_modified = true;
    }

    // ── Apply changes to GPS IFD ──────────────────────────────────────────────

    if let (Some(lat), Some(lon)) = (req.gps_latitude, req.gps_longitude) {
        // GPS Version ID (4 bytes, fits inline)
        let gps_ver = encode_bytes_inline(&[2, 3, 0, 0], bo);
        set_or_add_entry(&mut gps_ifd_entries, TAG_GPS_VERSION_ID, TYPE_BYTE, 4, gps_ver, bo);

        // Latitude reference
        let lat_ref_str = if lat >= 0.0 { "N\0" } else { "S\0" };
        let lat_ref_val = encode_ascii_inline(lat_ref_str, bo);
        set_or_add_entry(&mut gps_ifd_entries, TAG_GPS_LATITUDE_REF, TYPE_ASCII, 2, lat_ref_val, bo);

        // Latitude (3 RATIONALs = 24 bytes)
        let lat_offset = append_gps_dms(&mut data, lat.abs(), bo);
        set_or_add_entry(&mut gps_ifd_entries, TAG_GPS_LATITUDE, TYPE_RATIONAL, 3, lat_offset, bo);

        // Longitude reference
        let lon_ref_str = if lon >= 0.0 { "E\0" } else { "W\0" };
        let lon_ref_val = encode_ascii_inline(lon_ref_str, bo);
        set_or_add_entry(&mut gps_ifd_entries, TAG_GPS_LONGITUDE_REF, TYPE_ASCII, 2, lon_ref_val, bo);

        // Longitude (3 RATIONALs = 24 bytes)
        let lon_offset = append_gps_dms(&mut data, lon.abs(), bo);
        set_or_add_entry(&mut gps_ifd_entries, TAG_GPS_LONGITUDE, TYPE_RATIONAL, 3, lon_offset, bo);

        gps_ifd_modified = true;
    }

    // ── Rebuild modified IFDs and update pointers ─────────────────────────────

    // Write Exif sub-IFD if modified
    if exif_ifd_modified {
        exif_ifd_entries.sort_by_key(|e| e.tag);
        let exif_ifd_bytes = common::serialize_ifd(&exif_ifd_entries, 0, bo);
        let new_exif_ifd_offset = data.len() as u32;
        data.extend_from_slice(&exif_ifd_bytes);

        // Update or add Exif IFD pointer in IFD0
        set_or_add_entry(&mut ifd0_entries, TAG_EXIF_IFD_POINTER, 4 /* LONG */, 1, new_exif_ifd_offset, bo);
        ifd0_modified = true;
    }

    // Write GPS IFD if modified
    if gps_ifd_modified {
        gps_ifd_entries.sort_by_key(|e| e.tag);
        let gps_ifd_bytes = common::serialize_ifd(&gps_ifd_entries, 0, bo);
        let new_gps_ifd_offset = data.len() as u32;
        data.extend_from_slice(&gps_ifd_bytes);

        // Update or add GPS IFD pointer in IFD0
        set_or_add_entry(&mut ifd0_entries, TAG_GPS_IFD_POINTER, 4 /* LONG */, 1, new_gps_ifd_offset, bo);
        ifd0_modified = true;
    }

    // Rebuild IFD0 if modified
    if ifd0_modified {
        ifd0_entries.sort_by_key(|e| e.tag);
        let ifd0_bytes = common::serialize_ifd(&ifd0_entries, ifd0_next, bo);
        let new_ifd0_offset = data.len() as u32;
        data.extend_from_slice(&ifd0_bytes);

        // Update IFD0 offset in TIFF header
        common::write_u32(&mut data, 4, new_ifd0_offset, bo);
    }

    std::fs::write(file_path, &data)?;
    Ok(())
}

// ── IFD Manipulation Helpers ──────────────────────────────────────────────────

/// Set an existing IFD entry or add a new one.
fn set_or_add_entry(
    entries: &mut Vec<TiffIfdEntry>,
    tag: u16,
    field_type: u16,
    count: u32,
    value_offset: u32,
    _bo: TiffByteOrder,
) {
    if let Some(entry) = entries.iter_mut().find(|e| e.tag == tag) {
        entry.field_type = field_type;
        entry.count = count;
        entry.value_offset = value_offset;
    } else {
        entries.push(TiffIfdEntry {
            tag,
            field_type,
            count,
            value_offset,
        });
    }
}

/// Append raw data to the file buffer and return the offset where it was written.
fn append_data(data: &mut Vec<u8>, bytes: &[u8]) -> u32 {
    let offset = data.len() as u32;
    data.extend_from_slice(bytes);
    offset
}

/// Append a single RATIONAL value (num/denom) and return the offset.
fn append_rational(data: &mut Vec<u8>, num: u32, denom: u32, bo: TiffByteOrder) -> u32 {
    let offset = data.len() as u32;
    common::push_u32(data, num, bo);
    common::push_u32(data, denom, bo);
    offset
}

/// Append 3 RATIONAL values for GPS DMS and return the offset.
fn append_gps_dms(data: &mut Vec<u8>, decimal_degrees: f64, bo: TiffByteOrder) -> u32 {
    let degrees = decimal_degrees.floor();
    let minutes_full = (decimal_degrees - degrees) * 60.0;
    let minutes = minutes_full.floor();
    let seconds = (minutes_full - minutes) * 60.0;
    let sec_numerator = (seconds * 1_000_000.0).round() as u32;

    let offset = data.len() as u32;
    // Degrees
    common::push_u32(data, degrees as u32, bo);
    common::push_u32(data, 1, bo);
    // Minutes
    common::push_u32(data, minutes as u32, bo);
    common::push_u32(data, 1, bo);
    // Seconds
    common::push_u32(data, sec_numerator, bo);
    common::push_u32(data, 1_000_000, bo);
    offset
}

/// Encode a SHORT value for inline storage in value_offset field.
fn encode_short_inline(val: u16, bo: TiffByteOrder) -> u32 {
    match bo {
        TiffByteOrder::LittleEndian => val as u32,
        TiffByteOrder::BigEndian => (val as u32) << 16,
    }
}

/// Encode up to 4 bytes for inline storage.
fn encode_bytes_inline(bytes: &[u8], _bo: TiffByteOrder) -> u32 {
    let mut val = [0u8; 4];
    for (i, &b) in bytes.iter().take(4).enumerate() {
        val[i] = b;
    }
    u32::from_le_bytes(val)
}

/// Encode a short ASCII string (<=4 bytes including null) for inline storage.
fn encode_ascii_inline(s: &str, _bo: TiffByteOrder) -> u32 {
    let bytes = s.as_bytes();
    let mut val = [0u8; 4];
    for (i, &b) in bytes.iter().take(4).enumerate() {
        val[i] = b;
    }
    u32::from_le_bytes(val)
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

    fn write_and_readback_tiff(path: &Path, req: &ExifWriteRequest) -> ExifInfo {
        write_exif_fields_tiff(path, req).expect("write_exif_fields_tiff failed");
        read_exif_tiff(path).expect("read_exif_tiff failed")
    }

    #[test]
    fn test_read_exif_tiff() {
        let path = project_root().join("test_data").join("000226830002.tif");
        let result = read_exif_tiff(&path);
        // TIFF file should be readable (may or may not have EXIF)
        assert!(result.is_ok(), "Should be able to read TIFF file: {:?}", result.err());
    }

    #[test]
    fn test_write_and_read_rating_tiff() {
        let path = copy_test_file("000226830002.tif", "rating");
        let req = ExifWriteRequest {
            rating: Some(4),
            ..Default::default()
        };
        let info = write_and_readback_tiff(&path, &req);
        assert_eq!(info.rating, Some(4), "Rating should be 4");
    }

    #[test]
    fn test_write_and_read_datetime_tiff() {
        let path = copy_test_file("000226830002.tif", "datetime");
        let req = ExifWriteRequest {
            datetime: Some("2025:03:20 14:30:00".to_string()),
            ..Default::default()
        };
        let info = write_and_readback_tiff(&path, &req);
        assert!(
            info.datetime.as_ref().map_or(false, |dt| dt.contains("2025") && dt.contains("03") && dt.contains("20")),
            "DateTime mismatch: {:?}", info.datetime
        );
    }

    #[test]
    fn test_write_and_read_camera_model_tiff() {
        let path = copy_test_file("000226830002.tif", "cam_model");
        let req = ExifWriteRequest {
            camera_model: Some("TIFF Test Camera".to_string()),
            ..Default::default()
        };
        let info = write_and_readback_tiff(&path, &req);
        assert_eq!(info.camera_model.as_deref(), Some("TIFF Test Camera"));
    }

    #[test]
    fn test_write_and_read_iso_tiff() {
        let path = copy_test_file("000226830002.tif", "iso");
        let req = ExifWriteRequest {
            iso: Some(400),
            ..Default::default()
        };
        let info = write_and_readback_tiff(&path, &req);
        assert_eq!(info.iso, Some(400), "ISO should be 400");
    }

    #[test]
    fn test_write_and_read_aperture_tiff() {
        let path = copy_test_file("000226830002.tif", "aperture");
        let req = ExifWriteRequest {
            aperture: Some(5.6),
            ..Default::default()
        };
        let info = write_and_readback_tiff(&path, &req);
        assert!(info.aperture.is_some(), "Aperture should be set");
        let ap = info.aperture.unwrap();
        assert!((ap - 5.6).abs() < 0.1, "Aperture mismatch: {}", ap);
    }

    #[test]
    fn test_write_and_read_gps_tiff() {
        let path = copy_test_file("000226830002.tif", "gps");
        let req = ExifWriteRequest {
            gps_latitude: Some(51.5074),
            gps_longitude: Some(-0.1278),
            ..Default::default()
        };
        let info = write_and_readback_tiff(&path, &req);
        assert!(info.gps_latitude.is_some(), "GPS latitude should be set");
        assert!(info.gps_longitude.is_some(), "GPS longitude should be set");
        let lat = info.gps_latitude.unwrap();
        let lon = info.gps_longitude.unwrap();
        assert!((lat - 51.5074).abs() < 0.001, "Latitude mismatch: {}", lat);
        assert!((lon - (-0.1278)).abs() < 0.001, "Longitude mismatch: {}", lon);
    }

    #[test]
    fn test_write_multiple_fields_tiff() {
        let path = copy_test_file("000226830002.tif", "multi");
        let req = ExifWriteRequest {
            datetime: Some("2025:07:01 08:00:00".to_string()),
            camera_model: Some("MultiTest TIFF".to_string()),
            iso: Some(1600),
            rating: Some(5),
            ..Default::default()
        };
        let info = write_and_readback_tiff(&path, &req);
        assert!(info.datetime.as_ref().map_or(false, |dt| dt.contains("2025")));
        assert_eq!(info.camera_model.as_deref(), Some("MultiTest TIFF"));
        assert_eq!(info.iso, Some(1600));
        assert_eq!(info.rating, Some(5));
    }

    #[test]
    fn test_rating_validation_tiff() {
        let path = copy_test_file("000226830002.tif", "rating_invalid");
        let req = ExifWriteRequest {
            rating: Some(6),
            ..Default::default()
        };
        let result = write_exif_fields_tiff(&path, &req);
        assert!(result.is_err(), "Rating > 5 should be rejected");
    }
}
