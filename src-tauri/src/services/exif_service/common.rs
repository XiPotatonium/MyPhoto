use std::io::Cursor;

use exif::experimental::Writer as ExifWriter;
use exif::{Context, Field, In, Rational, Tag, Value};

use crate::models::exif::{ExifInfo, ExifWriteRequest};

// ── EXIF Read Helpers ─────────────────────────────────────────────────────────

/// Extract a text string from an EXIF field, properly handling UTF-8 encoded text.
///
/// The `exif` crate's `display_value()` escapes non-ASCII bytes (e.g. Chinese characters
/// become `\xe5\xb8\x83...`). This function reads the raw bytes and decodes them as UTF-8
/// to correctly handle multi-byte characters.
fn field_to_string(field: &exif::Field) -> Option<String> {
    match &field.value {
        exif::Value::Ascii(ref vec) => {
            for s in vec {
                let decoded = String::from_utf8_lossy(s).trim().to_string();
                if !decoded.is_empty() {
                    return Some(decoded);
                }
            }
            None
        }
        _ => {
            let s = field.display_value().to_string().trim_matches('"').trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        }
    }
}

/// Extract [`ExifInfo`] from already-parsed EXIF data.
/// Shared by all format-specific readers.
pub(super) fn parse_exif_data(exif_data: &exif::Exif) -> Result<ExifInfo, crate::error::AppError> {
    let mut info = ExifInfo::default();

    // DateTime
    if let Some(field) = exif_data.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        info.datetime = field_to_string(field);
    } else if let Some(field) = exif_data.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
        info.datetime = field_to_string(field);
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
        info.camera_make = field_to_string(field);
    }

    // Camera Model
    if let Some(field) = exif_data.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        info.camera_model = field_to_string(field);
    }

    // Lens Model & Make
    let lens_make = exif_data
        .get_field(exif::Tag::LensMake, exif::In::PRIMARY)
        .and_then(|f| field_to_string(f));
    let lens_model = exif_data
        .get_field(exif::Tag::LensModel, exif::In::PRIMARY)
        .and_then(|f| field_to_string(f));
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
pub(super) fn parse_gps_rational(value: &exif::Value) -> Option<f64> {
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
#[allow(dead_code)]
pub(super) fn parse_ascii_field(value: &exif::Value) -> Option<String> {
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

// ── EXIF Write Helpers ────────────────────────────────────────────────────────

/// Build the list of EXIF tags to replace and the new fields from a write request.
///
/// Shared by JPEG and RAF write paths. Pure data conversion, no I/O.
pub(super) fn build_exif_changes(
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
        new_fields.push(Field {
            tag: Tag(Context::Tiff, 0x4746),
            ifd_num: In::PRIMARY,
            value: Value::Short(vec![rating as u16]),
        });
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
pub(super) fn build_merged_exif_bytes(
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
pub(super) fn build_jpeg_with_exif(
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
            output.push(0xFF);
            output.push(marker);
        } else if (0xD0..=0xD7).contains(&marker) {
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

// ── Utility functions ─────────────────────────────────────────────────────────

/// Parse a shutter speed string into (numerator, denominator).
///
/// Accepts:
/// - `"1/500"` -> `(1, 500)`
/// - `"2"` or `"2.0"` -> `(2, 1)`
/// - `"0.5"` -> `(1, 2)`
pub(super) fn parse_shutter_speed(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    if let Some(slash_pos) = s.find('/') {
        let num: u32 = s[..slash_pos].trim().parse().ok()?;
        let denom: u32 = s[slash_pos + 1..].trim().parse().ok()?;
        if denom == 0 {
            return None;
        }
        Some((num, denom))
    } else {
        let secs: f64 = s.parse().ok()?;
        if secs <= 0.0 {
            return None;
        }
        if secs >= 1.0 {
            Some((secs.round() as u32, 1))
        } else {
            let denom = (1.0 / secs).round() as u32;
            Some((1, denom))
        }
    }
}

/// Convert absolute decimal degrees to 3 DMS RATIONAL values.
pub(super) fn decimal_degrees_to_dms(decimal: f64) -> Vec<Rational> {
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

// ── TIFF IFD Write Infrastructure ─────────────────────────────────────────────

/// Byte order for TIFF IFD operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum TiffByteOrder {
    LittleEndian,
    BigEndian,
}

/// A parsed IFD entry for TIFF write operations
#[derive(Debug, Clone)]
pub(super) struct TiffIfdEntry {
    pub tag: u16,
    pub field_type: u16,
    pub count: u32,
    pub value_offset: u32,
}

/// Read a u16 from data at a given offset, respecting byte order.
pub(super) fn read_u16(data: &[u8], offset: usize, bo: TiffByteOrder) -> u16 {
    match bo {
        TiffByteOrder::LittleEndian => u16::from_le_bytes([data[offset], data[offset + 1]]),
        TiffByteOrder::BigEndian => u16::from_be_bytes([data[offset], data[offset + 1]]),
    }
}

/// Read a u32 from data at a given offset, respecting byte order.
pub(super) fn read_u32(data: &[u8], offset: usize, bo: TiffByteOrder) -> u32 {
    match bo {
        TiffByteOrder::LittleEndian => {
            u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
        }
        TiffByteOrder::BigEndian => {
            u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
        }
    }
}

/// Write a u16 to data at a given offset, respecting byte order.
#[allow(dead_code)]
pub(super) fn write_u16(data: &mut Vec<u8>, offset: usize, val: u16, bo: TiffByteOrder) {
    let bytes = match bo {
        TiffByteOrder::LittleEndian => val.to_le_bytes(),
        TiffByteOrder::BigEndian => val.to_be_bytes(),
    };
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
}

/// Write a u32 to data at a given offset, respecting byte order.
pub(super) fn write_u32(data: &mut Vec<u8>, offset: usize, val: u32, bo: TiffByteOrder) {
    let bytes = match bo {
        TiffByteOrder::LittleEndian => val.to_le_bytes(),
        TiffByteOrder::BigEndian => val.to_be_bytes(),
    };
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
    data[offset + 2] = bytes[2];
    data[offset + 3] = bytes[3];
}

/// Append a u16 to data, respecting byte order.
pub(super) fn push_u16(data: &mut Vec<u8>, val: u16, bo: TiffByteOrder) {
    let bytes = match bo {
        TiffByteOrder::LittleEndian => val.to_le_bytes(),
        TiffByteOrder::BigEndian => val.to_be_bytes(),
    };
    data.extend_from_slice(&bytes);
}

/// Append a u32 to data, respecting byte order.
pub(super) fn push_u32(data: &mut Vec<u8>, val: u32, bo: TiffByteOrder) {
    let bytes = match bo {
        TiffByteOrder::LittleEndian => val.to_le_bytes(),
        TiffByteOrder::BigEndian => val.to_be_bytes(),
    };
    data.extend_from_slice(&bytes);
}

/// Parse the TIFF header from raw file data.
/// Returns (byte_order, ifd0_offset).
pub(super) fn parse_tiff_header(data: &[u8]) -> Result<(TiffByteOrder, u32), crate::error::AppError> {
    if data.len() < 8 {
        return Err(crate::error::AppError::General("File too small for TIFF header".to_string()));
    }

    let bo = match &data[0..2] {
        b"II" => TiffByteOrder::LittleEndian,
        b"MM" => TiffByteOrder::BigEndian,
        _ => return Err(crate::error::AppError::General("Invalid TIFF byte order marker".to_string())),
    };

    let version = read_u16(data, 2, bo);
    if version != 42 {
        return Err(crate::error::AppError::General(
            format!("Invalid TIFF version: {}, expected 42", version),
        ));
    }

    let ifd0_offset = read_u32(data, 4, bo);
    Ok((bo, ifd0_offset))
}

/// Parse an IFD at the given offset from raw data.
/// Returns (entries, next_ifd_offset).
pub(super) fn parse_ifd(data: &[u8], offset: u32, bo: TiffByteOrder) -> Result<(Vec<TiffIfdEntry>, u32), crate::error::AppError> {
    let offset = offset as usize;
    if offset + 2 > data.len() {
        return Err(crate::error::AppError::General("IFD offset beyond file bounds".to_string()));
    }

    let entry_count = read_u16(data, offset, bo) as usize;
    let entries_start = offset + 2;
    let entries_end = entries_start + entry_count * 12;

    if entries_end + 4 > data.len() {
        return Err(crate::error::AppError::General("IFD entries extend beyond file bounds".to_string()));
    }

    let mut entries = Vec::with_capacity(entry_count);
    for i in 0..entry_count {
        let e_offset = entries_start + i * 12;
        entries.push(TiffIfdEntry {
            tag: read_u16(data, e_offset, bo),
            field_type: read_u16(data, e_offset + 2, bo),
            count: read_u32(data, e_offset + 4, bo),
            value_offset: read_u32(data, e_offset + 8, bo),
        });
    }

    let next_ifd_offset = read_u32(data, entries_end, bo);
    Ok((entries, next_ifd_offset))
}

/// Serialize an IFD (entries + next_ifd_offset) into bytes.
pub(super) fn serialize_ifd(entries: &[TiffIfdEntry], next_ifd_offset: u32, bo: TiffByteOrder) -> Vec<u8> {
    let mut buf = Vec::with_capacity(2 + entries.len() * 12 + 4);
    push_u16(&mut buf, entries.len() as u16, bo);
    for entry in entries {
        push_u16(&mut buf, entry.tag, bo);
        push_u16(&mut buf, entry.field_type, bo);
        push_u32(&mut buf, entry.count, bo);
        push_u32(&mut buf, entry.value_offset, bo);
    }
    push_u32(&mut buf, next_ifd_offset, bo);
    buf
}
