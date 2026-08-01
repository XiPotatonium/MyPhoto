mod common;
pub(crate) mod jpg;
pub(crate) mod raf;
pub(crate) mod tiff;
pub(crate) mod dng;
pub(crate) mod bmp;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::exif::{ExifInfo, ExifWriteRequest};

// ── read ──────────────────────────────────────────────────────────────────────

/// Read EXIF info from any supported image file.
/// Dispatches to the appropriate format-specific reader based on file extension.
///
/// Image dimensions do not depend on EXIF: when the EXIF data lacks
/// PixelXDimension/PixelYDimension (or the file has no EXIF at all), the
/// dimensions are read from the image header itself as a fallback.
pub fn read_exif(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    let result = match ext.as_deref() {
        Some("raf") => raf::read_exif_raf(file_path),
        Some("dng") => dng::read_exif_dng(file_path),
        Some("tif") | Some("tiff") => tiff::read_exif_tiff(file_path),
        Some("bmp") => bmp::read_exif_bmp(file_path),
        _ => jpg::read_exif_jpg(file_path),
    };

    let mut info = match result {
        Ok(info) => info,
        // 文件没有 EXIF 数据时不整体失败，仍返回带尺寸兜底的基础信息
        Err(crate::error::AppError::Exif(_)) => ExifInfo::default(),
        Err(e) => return Err(e),
    };

    if info.image_width.is_none() || info.image_height.is_none() {
        if let Some((width, height)) = read_image_dimensions(file_path, ext.as_deref()) {
            info.image_width = Some(width);
            info.image_height = Some(height);
        }
    }

    // Populate file creation time from filesystem metadata
    if let Ok(metadata) = std::fs::metadata(file_path) {
        if let Ok(created) = metadata.created() {
            let datetime = chrono::DateTime::<chrono::Local>::from(created);
            info.file_created_time = Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
        }
    }

    Ok(info)
}

/// 从图像文件本身读取像素尺寸（仅解析文件头，不解码像素数据）。
/// 作为 EXIF 中缺少尺寸标签时的兜底来源。
fn read_image_dimensions(file_path: &Path, ext: Option<&str>) -> Option<(u32, u32)> {
    use std::io::Cursor;

    match ext {
        // RAF：image crate 不支持该容器，仅读内嵌 JPEG 前缀从头部解析尺寸
        Some("raf") => {
            use crate::services::raw_decoders::raf_decoder::{JpegData, RafHeader};
            const RAF_PREFIX_LEN: usize = 512 * 1024;
            let header = RafHeader::parse(file_path).ok()?;
            let jpeg = JpegData::extract_prefix(
                file_path,
                header.offset_jpg_offset,
                header.offset_jpg_length,
                RAF_PREFIX_LEN,
            )
            .ok()?;
            let reader = image::ImageReader::new(Cursor::new(&jpeg.data))
                .with_guessed_format()
                .ok()?;
            reader.into_dimensions().ok()
        }
        // DNG：仅解析 IFD 链，优先用 tag 256/257 的预览尺寸，其次从预览前缀读取
        Some("dng") => {
            use crate::services::raw_decoders::dng_decoder::{
                DngDecoder, DngIfdChain, DngPreviewData,
            };
            const DNG_PREFIX_LEN: usize = 512 * 1024;
            let DngIfdChain {
                header,
                main_ifd,
                sub_ifds,
            } = DngDecoder::parse_ifds(file_path).ok()?;
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
                if preview.width > 0 && preview.height > 0 {
                    return Some((preview.width, preview.height));
                }
                if !preview.data.is_empty() {
                    if let Ok(reader) = image::ImageReader::new(Cursor::new(&preview.data))
                        .with_guessed_format()
                    {
                        if let Ok(dims) = reader.into_dimensions() {
                            return Some(dims);
                        }
                    }
                }
            }
            None
        }
        // 标准格式（JPG/PNG/TIFF/BMP 等）：image crate 直接读取文件头
        _ => {
            let reader = image::ImageReader::open(file_path)
                .ok()?
                .with_guessed_format()
                .ok()?;
            reader.into_dimensions().ok()
        }
    }
}

// ── write EXIF fields ─────────────────────────────────────────────────────────

/// Write EXIF fields to one or more image files in parallel.
///
/// Only fields with `Some` value in `req` are written; the rest are preserved.
/// When multiple file paths are provided, they are processed concurrently using
/// a thread pool sized to the number of available CPU cores.
///
/// Supported formats: JPEG, RAF, DNG, TIFF.
/// BMP format does not support EXIF and will return an error.
pub fn write_exif_fields(
    file_paths: &[String],
    req: &ExifWriteRequest,
) -> Result<(), crate::error::AppError> {
    // Separate paths by format type
    let mut jpg_paths: Vec<&String> = Vec::new();
    let mut raf_paths: Vec<&String> = Vec::new();
    let mut tiff_paths: Vec<&String> = Vec::new();
    let mut dng_paths: Vec<&String> = Vec::new();
    let mut unsupported_write_paths: Vec<&String> = Vec::new();

    for p in file_paths {
        let ext = Path::new(p)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        match ext.as_deref() {
            Some("raf") => raf_paths.push(p),
            Some("tif") | Some("tiff") => tiff_paths.push(p),
            Some("dng") => dng_paths.push(p),
            Some("bmp") => unsupported_write_paths.push(p),
            _ => jpg_paths.push(p),
        }
    }

    // BMP does not support EXIF write
    if !unsupported_write_paths.is_empty() {
        return Err(crate::error::AppError::General(format!(
            "EXIF write not supported for BMP format: {}",
            unsupported_write_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        )));
    }

    // Process RAF files
    for raf_path in &raf_paths {
        let path = Path::new(raf_path.as_str());
        raf::write_exif_fields_raf(path, req)?;
    }

    // Process TIFF files
    for tiff_path in &tiff_paths {
        let path = Path::new(tiff_path.as_str());
        tiff::write_exif_fields_tiff(path, req)?;
    }

    // Process DNG files
    for dng_path in &dng_paths {
        let path = Path::new(dng_path.as_str());
        dng::write_exif_fields_dng(path, req)?;
    }

    if jpg_paths.is_empty() {
        return Ok(());
    }

    if jpg_paths.len() == 1 {
        // Single file — skip thread overhead
        let path = Path::new(jpg_paths[0].as_str());
        return jpg::write_exif_fields_jpg(path, req);
    }

    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(jpg_paths.len());

    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let chunk_size = (jpg_paths.len() + num_threads - 1) / num_threads;
    let mut handles = vec![];

    for chunk in jpg_paths.chunks(chunk_size) {
        let chunk_owned: Vec<String> = chunk.iter().map(|s| s.to_string()).collect();
        let errors_clone = Arc::clone(&errors);
        let req_clone = req.clone();

        let handle = thread::spawn(move || {
            for file_path_str in chunk_owned {
                let path = Path::new(&file_path_str);
                if let Err(e) = jpg::write_exif_fields_jpg(path, &req_clone) {
                    let mut errs = errors_clone.lock().unwrap();
                    errs.push(format!("{}: {}", file_path_str, e));
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().map_err(|_| {
            crate::error::AppError::General("Thread panicked during EXIF write".to_string())
        })?;
    }

    let errors = errors.lock().unwrap();
    if !errors.is_empty() {
        return Err(crate::error::AppError::General(format!(
            "Failed to write EXIF to {} file(s): {}",
            errors.len(),
            errors.join("; ")
        )));
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_read_exif_dimensions_fallback_without_exif() {
        // 生成一张不含 EXIF 的 PNG，验证尺寸从图像文件头兜底读取
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_output");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("no_exif_dims.png");
        image::DynamicImage::new_rgb8(320, 240).save(&path).unwrap();

        let info = read_exif(&path).expect("read_exif should not fail for file without EXIF");
        assert_eq!(info.image_width, Some(320));
        assert_eq!(info.image_height, Some(240));
    }

    #[test]
    fn test_read_exif_dimensions_from_raf_fallback_helper() {
        // RAF 内嵌 JPEG 尺寸可通过兜底函数直接读取
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("sample.RAF");
        let dims = read_image_dimensions(&path, Some("raf"));
        assert!(dims.is_some(), "RAF dimensions fallback should succeed");
        let (w, h) = dims.unwrap();
        assert!(w > 0 && h > 0, "Dimensions should be positive: {}x{}", w, h);
    }
}
