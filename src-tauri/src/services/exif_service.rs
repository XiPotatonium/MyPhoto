use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::exif::ExifInfo;

// little_exif imports for writing rating
use little_exif::metadata::Metadata;
use little_exif::exif_tag::{ExifTag, ExifTagGroup};
use little_exif::exif_tag_format::ExifTagFormat;
use little_exif::endian::Endian;

// kamadak-exif imports for writing GPS
use exif::experimental::Writer as ExifWriter;
use exif::{Field, In, Rational, Tag, Value};

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
    // kamadak-exif supports reading custom tags using Tag::Unknown
    if let Some(field) = exif_data.get_field(exif::Tag(exif::Context::Tiff, 0x4746), exif::In::PRIMARY) {
        if let exif::Value::Short(ref v) = field.value {
            if let Some(&val) = v.first() {
                // Rating is 0-5
                if val <= 5 {
                    info.rating = Some(val as u8);
                }
            }
        }
    }

    // for field in exif_data.fields() {
    //     println!("{:?}: {:?}", field.tag.number(), field.display_value().to_string());
    // }

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

pub fn write_rating(file_path: &Path, rating: u8) -> Result<(), crate::error::AppError> {
    // Validate rating range (0-5)
    if rating > 5 {
        return Err(crate::error::AppError::General(
            "Rating must be between 0 and 5".to_string(),
        ));
    }

    // Check if file exists
    if !file_path.exists() {
        return Err(crate::error::AppError::General(
            "File not found".to_string(),
        ));
    }

    // Load existing metadata or create new
    let mut metadata = Metadata::new_from_path(file_path).map_err(|e| {
        crate::error::AppError::Exif(format!("Failed to read metadata: {}", e))
    })?;

    // Set the Rating tag (0x4746 in IFD0)
    // Rating 0 means "not rated" or "rejected" depending on the application
    // Using from_u16_with_data to create the tag since Rating is not a predefined variant
    // Convert u16 rating to little-endian bytes
    let rating_bytes = (rating as u16).to_le_bytes().to_vec();
    let rating_tag = ExifTag::from_u16_with_data(
        0x4746,
        &ExifTagFormat::INT16U,
        &rating_bytes,
        &Endian::Little,
        &ExifTagGroup::IFD0,
    )
    .map_err(|e| crate::error::AppError::Exif(format!("Failed to create rating tag: {}", e)))?;
    metadata.set_tag(rating_tag);

    // Also set the RatingPercent tag (0x4749) for compatibility
    // Convert 0-5 rating to percentage (0, 20, 40, 60, 80, 100)
    let percent = (rating as u16) * 20;
    let percent_bytes = percent.to_le_bytes().to_vec();
    let rating_percent_tag = ExifTag::from_u16_with_data(
        0x4749,
        &ExifTagFormat::INT16U,
        &percent_bytes,
        &Endian::Little,
        &ExifTagGroup::IFD0,
    )
    .map_err(|e| {
        crate::error::AppError::Exif(format!("Failed to create rating percent tag: {}", e))
    })?;
    metadata.set_tag(rating_percent_tag);

    // Write metadata back to file
    metadata.write_to_file(file_path).map_err(|e| {
        crate::error::AppError::Exif(format!("Failed to write metadata: {}", e))
    })?;

    Ok(())
}

pub fn batch_write_gps(
    file_paths: &[String],
    latitude: f64,
    longitude: f64,
) -> Result<(), crate::error::AppError> {
    // Get the number of available CPU cores
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    
    // Store errors from threads
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    
    // Split file paths into chunks for each thread
    let chunk_size = (file_paths.len() + num_threads - 1) / num_threads;
    let mut handles = vec![];
    
    for chunk in file_paths.chunks(chunk_size) {
        let chunk_owned = chunk.to_vec();
        let errors_clone = Arc::clone(&errors);
        
        let handle = thread::spawn(move || {
            for file_path_str in chunk_owned {
                if let Err(e) = write_gps_to_file(&file_path_str, latitude, longitude) {
                    let mut errs = errors_clone.lock().unwrap();
                    errs.push(format!("{}: {}", file_path_str, e));
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().map_err(|_| {
            crate::error::AppError::General("Thread panicked during GPS write".to_string())
        })?;
    }
    
    // Check if there were any errors
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

/// Write GPS coordinates to a single image file using kamadak-exif.
///
/// Writes the following EXIF GPS tags:
///   - GPSVersionID  (0x0000) — version 2.3.0.0
///   - GPSLatitudeRef  (0x0001) — "N" or "S"
///   - GPSLatitude     (0x0002) — [deg, min, sec] as RATIONAL
///   - GPSLongitudeRef (0x0003) — "E" or "W"
///   - GPSLongitude    (0x0004) — [deg, min, sec] as RATIONAL
fn write_gps_to_file(
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

    // Create GPS fields (these need to live long enough for the writer)
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
    
    // Create writer and add GPS fields
    let mut exif_writer = ExifWriter::new();
    exif_writer.push_field(&gps_version_field);
    exif_writer.push_field(&lat_ref_field);
    exif_writer.push_field(&lat_field);
    exif_writer.push_field(&lon_ref_field);
    exif_writer.push_field(&lon_field);
    
    // Write EXIF data to a buffer (little_endian = false means big-endian, which is standard for JPEG)
    let mut output_buf = Cursor::new(Vec::new());
    exif_writer
        .write(&mut output_buf, false)
        .map_err(|e| crate::error::AppError::Exif(format!("Failed to write EXIF: {}", e)))?;

    // For JPEG files, we need to merge the new EXIF data with the original file
    // The simplest approach is to read existing EXIF, add GPS fields, and write back
    // However, the Writer doesn't support reading + merging, so we need a different approach
    
    // Read existing EXIF data if available
    let file = File::open(file_path)?;
    let mut bufreader = BufReader::new(&file);
    let exif_reader = exif::Reader::new();
    
    // Try to read existing EXIF; if it fails, we'll write new EXIF
    match exif_reader.read_from_container(&mut bufreader) {
        Ok(existing_exif) => {
            // Create a new writer with both existing and new GPS fields
            let mut merged_writer = ExifWriter::new();
            
            // First, add all existing fields except GPS-related ones
            for field in existing_exif.fields() {
                // Skip GPS fields as we'll add new ones
                if !matches!(field.tag, 
                    Tag::GPSVersionID | Tag::GPSLatitudeRef | Tag::GPSLatitude | 
                    Tag::GPSLongitudeRef | Tag::GPSLongitude
                ) {
                    merged_writer.push_field(field);
                }
            }
            
            // Add new GPS fields
            merged_writer.push_field(&gps_version_field);
            merged_writer.push_field(&lat_ref_field);
            merged_writer.push_field(&lat_field);
            merged_writer.push_field(&lon_ref_field);
            merged_writer.push_field(&lon_field);
            
            // Write merged EXIF to buffer
            let mut merged_buf = Cursor::new(Vec::new());
            merged_writer
                .write(&mut merged_buf, false)
                .map_err(|e| crate::error::AppError::Exif(format!("Failed to write merged EXIF: {}", e)))?;
            
            // Replace EXIF segment in JPEG file
            let exif_data = merged_buf.into_inner();
            write_jpeg_with_exif(file_path, &original_data, &exif_data)?;
        },
        Err(_) => {
            // No existing EXIF, write new EXIF data
            let exif_data = output_buf.into_inner();
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

    // High-precision seconds: numerator = round(sec × 1 000 000)
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
fn write_jpeg_with_exif(
    file_path: &Path,
    original_data: &[u8],
    exif_data: &[u8],
) -> Result<(), crate::error::AppError> {
    // JPEG markers
    const SOI: [u8; 2] = [0xFF, 0xD8];  // Start of Image
    const APP1: u8 = 0xE1;               // APP1 marker
    const EXIF_HEADER: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00]; // "Exif\0\0"

    if original_data.len() < 2 || &original_data[0..2] != &SOI {
        return Err(crate::error::AppError::General(
            "Not a valid JPEG file".to_string(),
        ));
    }

    let mut output = Vec::new();
    output.extend_from_slice(&SOI);

    // Build the new APP1 EXIF segment
    let exif_segment_size = 2 + 6 + exif_data.len(); // size field (2) + "Exif\0\0" (6) + EXIF data
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

    // Parse the rest of the original JPEG and skip existing EXIF APP1 segment
    let mut pos = 2; // Skip SOI

    while pos + 1 < original_data.len() {
        if original_data[pos] != 0xFF {
            break; // Not a marker, rest is image data
        }

        let marker = original_data[pos + 1];
        pos += 2;

        // Check if this is an APP1 EXIF segment to skip
        if marker == APP1 && pos + 2 <= original_data.len() {
            let seg_size = u16::from_be_bytes([original_data[pos], original_data[pos + 1]]) as usize;
            
            // Check if it's an EXIF segment
            if pos + seg_size <= original_data.len() 
                && seg_size >= 8 
                && &original_data[pos + 2..pos + 8] == &EXIF_HEADER 
            {
                // Skip this EXIF segment
                pos += seg_size;
                continue;
            }
        }

        // For other segments, copy them
        if marker == 0xD8 || marker == 0xD9 { // SOI or EOI (no size field)
            output.push(0xFF);
            output.push(marker);
        } else if marker >= 0xD0 && marker <= 0xD7 { // RST markers (no size field)
            output.push(0xFF);
            output.push(marker);
        } else if pos + 2 <= original_data.len() {
            // Marker with size field
            let seg_size = u16::from_be_bytes([original_data[pos], original_data[pos + 1]]) as usize;
            if pos + seg_size <= original_data.len() {
                output.push(0xFF);
                output.push(marker);
                output.extend_from_slice(&original_data[pos..pos + seg_size]);
                pos += seg_size;
            } else {
                break; // Invalid segment size
            }
        } else {
            break; // Incomplete segment header
        }
    }

    // Copy the rest of the file (compressed image data)
    if pos < original_data.len() {
        output.extend_from_slice(&original_data[pos..]);
    }

    // Write the modified JPEG to disk
    std::fs::write(file_path, &output)?;

    Ok(())
}
