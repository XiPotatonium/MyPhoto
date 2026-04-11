use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use exif::experimental::Writer as ExifWriter;
use exif::{Field, In, Rational, Tag, Value};

/// Write GPS coordinates to all files in parallel.
///
/// Currently only JPEG files are supported; RAF support is not yet implemented.
pub fn batch_write_gps(
    file_paths: &[String],
    latitude: f64,
    longitude: f64,
) -> Result<(), crate::error::AppError> {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let chunk_size = (file_paths.len() + num_threads - 1) / num_threads;
    let mut handles = vec![];

    for chunk in file_paths.chunks(chunk_size) {
        let chunk_owned = chunk.to_vec();
        let errors_clone = Arc::clone(&errors);

        let handle = thread::spawn(move || {
            for file_path_str in chunk_owned {
                if let Err(e) = write_gps_jpg(&file_path_str, latitude, longitude) {
                    let mut errs = errors_clone.lock().unwrap();
                    errs.push(format!("{}: {}", file_path_str, e));
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().map_err(|_| {
            crate::error::AppError::General("Thread panicked during GPS write".to_string())
        })?;
    }

    let errors = errors.lock().unwrap();
    if !errors.is_empty() {
        return Err(crate::error::AppError::General(format!(
            "Failed to write GPS data to {} file(s): {}",
            errors.len(),
            errors.join("; ")
        )));
    }

    Ok(())
}

/// Write GPS coordinates to a single JPEG file using kamadak-exif.
///
/// Writes the following EXIF GPS tags:
///   - GPSVersionID  (0x0000) — version 2.3.0.0
///   - GPSLatitudeRef  (0x0001) — "N" or "S"
///   - GPSLatitude     (0x0002) — [deg, min, sec] as RATIONAL
///   - GPSLongitudeRef (0x0003) — "E" or "W"
///   - GPSLongitude    (0x0004) — [deg, min, sec] as RATIONAL
fn write_gps_jpg(
    file_path_str: &str,
    latitude: f64,
    longitude: f64,
) -> Result<(), crate::error::AppError> {
    let file_path = Path::new(file_path_str);

    if !file_path.exists() {
        return Err(crate::error::AppError::General(format!(
            "File not found: {}",
            file_path_str
        )));
    }

    // Hemisphere references
    let lat_ref = if latitude >= 0.0 { "N" } else { "S" };
    let lon_ref = if longitude >= 0.0 { "E" } else { "W" };

    // Convert decimal degrees → DMS rationals
    let lat_dms = decimal_degrees_to_dms(latitude.abs());
    let lon_dms = decimal_degrees_to_dms(longitude.abs());

    // Create GPS fields
    let gps_version_field = Field {
        tag: Tag::GPSVersionID,
        ifd_num: In::PRIMARY,
        value: Value::Byte(vec![2, 3, 0, 0]),
    };

    let lat_ref_field = Field {
        tag: Tag::GPSLatitudeRef,
        ifd_num: In::PRIMARY,
        value: Value::Ascii(vec![lat_ref.as_bytes().to_vec()]),
    };

    let lat_field = Field {
        tag: Tag::GPSLatitude,
        ifd_num: In::PRIMARY,
        value: Value::Rational(lat_dms),
    };

    let lon_ref_field = Field {
        tag: Tag::GPSLongitudeRef,
        ifd_num: In::PRIMARY,
        value: Value::Ascii(vec![lon_ref.as_bytes().to_vec()]),
    };

    let lon_field = Field {
        tag: Tag::GPSLongitude,
        ifd_num: In::PRIMARY,
        value: Value::Rational(lon_dms),
    };

    // Read original file data
    let original_data = std::fs::read(file_path)?;

    // Build the GPS-only EXIF buffer (fallback when no existing EXIF)
    let mut fallback_writer = ExifWriter::new();
    fallback_writer.push_field(&gps_version_field);
    fallback_writer.push_field(&lat_ref_field);
    fallback_writer.push_field(&lat_field);
    fallback_writer.push_field(&lon_ref_field);
    fallback_writer.push_field(&lon_field);
    let mut fallback_buf = Cursor::new(Vec::new());
    fallback_writer
        .write(&mut fallback_buf, false)
        .map_err(|e| crate::error::AppError::Exif(format!("Failed to write EXIF: {}", e)))?;

    // Try to read existing EXIF and merge
    let file = File::open(file_path)?;
    let mut bufreader = BufReader::new(&file);
    let exif_reader = exif::Reader::new();

    match exif_reader.read_from_container(&mut bufreader) {
        Ok(existing_exif) => {
            // Merge: keep all non-GPS fields, then add new GPS fields
            let mut merged_writer = ExifWriter::new();

            for field in existing_exif.fields() {
                if !matches!(
                    field.tag,
                    Tag::GPSVersionID
                        | Tag::GPSLatitudeRef
                        | Tag::GPSLatitude
                        | Tag::GPSLongitudeRef
                        | Tag::GPSLongitude
                ) {
                    merged_writer.push_field(field);
                }
            }

            merged_writer.push_field(&gps_version_field);
            merged_writer.push_field(&lat_ref_field);
            merged_writer.push_field(&lat_field);
            merged_writer.push_field(&lon_ref_field);
            merged_writer.push_field(&lon_field);

            let mut merged_buf = Cursor::new(Vec::new());
            merged_writer
                .write(&mut merged_buf, false)
                .map_err(|e| crate::error::AppError::Exif(format!("Failed to write merged EXIF: {}", e)))?;

            let exif_data = merged_buf.into_inner();
            write_jpeg_with_exif(file_path, &original_data, &exif_data)?;
        }
        Err(_) => {
            let exif_data = fallback_buf.into_inner();
            write_jpeg_with_exif(file_path, &original_data, &exif_data)?;
        }
    }

    Ok(())
}

/// Convert absolute decimal degrees to 3 EXIF GPS RATIONAL values (DMS format):
/// `[degrees/1, minutes/1, seconds×1000000/1000000]`
///
/// Seconds use a denominator of 1 000 000 to preserve ~0.00003 arc-second
/// precision (≈ 1 mm on the ground).
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

/// Write EXIF data into a JPEG file by replacing or adding the APP1 EXIF segment.
///
/// This function handles JPEG marker structure properly:
/// - Preserves SOI (Start of Image) marker
/// - Replaces existing APP1 EXIF segment or inserts new one after SOI
/// - Preserves all other segments and image data
pub(super) fn write_jpeg_with_exif(
    file_path: &Path,
    original_data: &[u8],
    exif_data: &[u8],
) -> Result<(), crate::error::AppError> {
    const SOI: [u8; 2] = [0xFF, 0xD8];
    const APP1: u8 = 0xE1;
    const EXIF_HEADER: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00]; // "Exif\0\0"

    if original_data.len() < 2 || original_data[0..2] != SOI {
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

    while pos + 1 < original_data.len() {
        if original_data[pos] != 0xFF {
            break;
        }

        let marker = original_data[pos + 1];
        pos += 2;

        // Skip existing APP1 EXIF segment
        if marker == APP1 && pos + 2 <= original_data.len() {
            let seg_size =
                u16::from_be_bytes([original_data[pos], original_data[pos + 1]]) as usize;

            if pos + seg_size <= original_data.len()
                && seg_size >= 8
                && original_data[pos + 2..pos + 8] == EXIF_HEADER
            {
                pos += seg_size;
                continue;
            }
        }

        // Copy other segments as-is
        if marker == 0xD8 || marker == 0xD9 {
            // SOI / EOI — no size field
            output.push(0xFF);
            output.push(marker);
        } else if (0xD0..=0xD7).contains(&marker) {
            // RST markers — no size field
            output.push(0xFF);
            output.push(marker);
        } else if pos + 2 <= original_data.len() {
            let seg_size =
                u16::from_be_bytes([original_data[pos], original_data[pos + 1]]) as usize;
            if pos + seg_size <= original_data.len() {
                output.push(0xFF);
                output.push(marker);
                output.extend_from_slice(&original_data[pos..pos + seg_size]);
                pos += seg_size;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if pos < original_data.len() {
        output.extend_from_slice(&original_data[pos..]);
    }

    std::fs::write(file_path, &output)?;

    Ok(())
}
