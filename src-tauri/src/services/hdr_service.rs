//! TIF / BMP → Ultra HDR JPEG（ISO 21496-1 / Google Ultra HDR）转换。
//!
//! 转换逻辑参考 gainmap CLI（https://github.com/kirkstrobeck/gainmaps.com）的
//! keep-base 编码器：
//! 1. SDR 基础图 = 原始像素（不做色调映射），因此 SDR 显示设备上看到的画面与
//!    原 TIF/BMP 一致；
//! 2. 按亮度做 highlight-selective 增益，把增益量化成增益图（gain map）；
//! 3. 基础图与增益图分别编码为 JPEG，再拼成一个 Ultra HDR JPEG：
//!    EXIF(APP1) + XMP(APP1) + MPF(APP2) + 基础图 + 增益图（第二张 JPEG）。
//!
//! 为了让转换后的 JPG 仍能被“照片信息”面板读到评级 / 拍摄参数，源 TIF 的常用
//! EXIF 字段会被复制进输出的 EXIF APP1 段。

use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ExtendedColorType, ImageDecoder, ImageReader, RgbImage};

use crate::error::AppError;
use crate::services::exif_service;

/// 与 gainmap CLI 文档默认值 boost=0.5 对齐的 HDR 增益上限（约 3.34×）。
/// 只影响 HDR 屏幕上的高光亮度，不影响 SDR 基础图。
const DEFAULT_HEADROOM: f64 = 3.3431570529937744;
/// 基础图 / 增益图 JPEG 质量，与 gainmap CLI 默认值一致。
const DEFAULT_JPEG_QUALITY: u8 = 92;
/// ISO 21496-1 建议的线性域偏移，避免暗部除零。
const OFFSET_SDR: f64 = 1.0 / 64.0;
const OFFSET_HDR: f64 = 1.0 / 64.0;
/// highlight-selective 模型的亮度过渡区间。
const HIGHLIGHT_EDGE_LOW: f64 = 0.28;
const HIGHLIGHT_EDGE_HIGH: f64 = 0.88;

/// XMP APP1 命名空间（含结尾 NUL），与 gainmap CLI 输出一致。
const XMP_NAMESPACE: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";
/// 与 gainmap CLI（hdrify 默认值）一致的参考 sRGB ICC profile。
/// 来源：hdrify-icc（MIT）中的 Google Inc. 2016 sRGB display profile。
/// 基础图与增益图都带上该 profile，提升 Apple Preview / Android 等 HDR 阅读器的兼容性。
const SRGB_ICC_PROFILE: &[u8] = &[
    0x00, 0x00, 0x01, 0xc8, 0x00, 0x00, 0x00, 0x00, 0x04, 0x30, 0x00, 0x00, 0x6d, 0x6e, 0x74, 0x72,
    0x52, 0x47, 0x42, 0x20, 0x58, 0x59, 0x5a, 0x20, 0x07, 0xe0, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x61, 0x63, 0x73, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xf6, 0xd6, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0xd3, 0x2d,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x09, 0x64, 0x65, 0x73, 0x63, 0x00, 0x00, 0x00, 0xf0, 0x00, 0x00, 0x00, 0x24,
    0x72, 0x58, 0x59, 0x5a, 0x00, 0x00, 0x01, 0x14, 0x00, 0x00, 0x00, 0x14, 0x67, 0x58, 0x59, 0x5a,
    0x00, 0x00, 0x01, 0x28, 0x00, 0x00, 0x00, 0x14, 0x62, 0x58, 0x59, 0x5a, 0x00, 0x00, 0x01, 0x3c,
    0x00, 0x00, 0x00, 0x14, 0x77, 0x70, 0x70, 0x74, 0x00, 0x00, 0x01, 0x50, 0x00, 0x00, 0x00, 0x14,
    0x72, 0x54, 0x52, 0x43, 0x00, 0x00, 0x01, 0x64, 0x00, 0x00, 0x00, 0x28, 0x67, 0x54, 0x52, 0x43,
    0x00, 0x00, 0x01, 0x64, 0x00, 0x00, 0x00, 0x28, 0x62, 0x54, 0x52, 0x43, 0x00, 0x00, 0x01, 0x64,
    0x00, 0x00, 0x00, 0x28, 0x63, 0x70, 0x72, 0x74, 0x00, 0x00, 0x01, 0x8c, 0x00, 0x00, 0x00, 0x3c,
    0x6d, 0x6c, 0x75, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0c,
    0x65, 0x6e, 0x55, 0x53, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x73, 0x00, 0x52,
    0x00, 0x47, 0x00, 0x42, 0x58, 0x59, 0x5a, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6f, 0xa2,
    0x00, 0x00, 0x38, 0xf5, 0x00, 0x00, 0x03, 0x90, 0x58, 0x59, 0x5a, 0x20, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x62, 0x99, 0x00, 0x00, 0xb7, 0x85, 0x00, 0x00, 0x18, 0xda, 0x58, 0x59, 0x5a, 0x20,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0xa0, 0x00, 0x00, 0x0f, 0x84, 0x00, 0x00, 0xb6, 0xcf,
    0x58, 0x59, 0x5a, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0xd6, 0x00, 0x01, 0x00, 0x00,
    0x00, 0x00, 0xd3, 0x2d, 0x70, 0x61, 0x72, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00,
    0x00, 0x02, 0x66, 0x66, 0x00, 0x00, 0xf2, 0xa7, 0x00, 0x00, 0x0d, 0x59, 0x00, 0x00, 0x13, 0xd0,
    0x00, 0x00, 0x0a, 0x5b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6d, 0x6c, 0x75, 0x63,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0c, 0x65, 0x6e, 0x55, 0x53,
    0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x47, 0x00, 0x6f, 0x00, 0x6f, 0x00, 0x67,
    0x00, 0x6c, 0x00, 0x65, 0x00, 0x20, 0x00, 0x49, 0x00, 0x6e, 0x00, 0x63, 0x00, 0x2e, 0x00, 0x20,
    0x00, 0x32, 0x00, 0x30, 0x00, 0x31, 0x00, 0x36,
];
/// MPF 结构固定长度：签名 4 + 端序 2 + magic 2 + IFD 偏移 4 + 条目数 2
/// + 3×12 标签 + 下一个 IFD 偏移 4 + 2×16 MP Entry。
const MPF_SIZE: usize = 86;

/// JPEG marker 常量。
const MARKER_SOI: u8 = 0xD8;
const MARKER_APP1: u8 = 0xE1;
const MARKER_APP2: u8 = 0xE2;

pub struct HdrConvertOutcome {
    pub output_path: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// 只有 TIF / TIFF / BMP 支持转换为 HDR。
pub fn is_convertible_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref(),
        Some("tif") | Some("tiff") | Some("bmp")
    )
}

/// 输出文件名：同目录、同主文件名、后缀改为小写 .jpg。
pub fn output_path_for(input: &Path) -> Option<PathBuf> {
    let stem = input.file_stem()?.to_str()?;
    if stem.is_empty() {
        return None;
    }
    Some(input.with_file_name(format!("{stem}.jpg")))
}

/// 把单个 TIF/BMP 转换为 Ultra HDR JPEG，输出到原图同目录（同名 .jpg 会被覆盖）。
pub fn convert_file_to_hdr(
    input: &Path,
    quality: Option<u8>,
) -> Result<HdrConvertOutcome, AppError> {
    if !is_convertible_file(input) {
        return Err(AppError::General(format!(
            "不支持转换为 HDR 的格式: {}",
            input.display()
        )));
    }
    let output_path = output_path_for(input).ok_or_else(|| {
        AppError::General(format!("无法从文件名推导输出路径: {}", input.display()))
    })?;

    let rgb = decode_to_rgb8(input)?;
    let (width, height) = rgb.dimensions();
    if width == 0 || height == 0 {
        return Err(AppError::General(format!(
            "图像尺寸无效: {}x{}",
            width, height
        )));
    }

    let quality = quality.unwrap_or(DEFAULT_JPEG_QUALITY).clamp(1, 100);
    let (gain_map, max_log2) = build_gain_map(&rgb, DEFAULT_HEADROOM);
    let sdr_jpeg = encode_jpeg(rgb.as_raw(), width, height, quality)?;
    let gain_jpeg = encode_jpeg(&gain_map, width, height, quality)?;

    // 复制源文件常用 EXIF，避免转换后照片信息面板丢失评级 / 拍摄参数。
    // 覆盖已有同名 JPG 时，源图缺失的字段用旧 JPG 的值补齐（例如用户后来打的星级评分）。
    let exif_payload = build_output_exif(input, &output_path);

    let bytes = assemble_ultra_hdr_jpeg(&sdr_jpeg, &gain_jpeg, max_log2, exif_payload.as_deref())?;
    atomic_write(&output_path, &bytes)?;

    Ok(HdrConvertOutcome {
        output_path,
        width,
        height,
    })
}

/// 组装输出 JPG 的 EXIF APP1 负载。
///
/// 以源 TIF/BMP 的 EXIF 为主；如果输出位置已存在同名 JPG（即将被覆盖），
/// 则用旧 JPG 的字段补齐源图缺失的部分，避免用户后加的评分 / 拍摄信息丢失。
fn build_output_exif(input: &Path, output: &Path) -> Option<Vec<u8>> {
    let mut info = exif_service::read_exif(input).unwrap_or_default();

    if output.exists() {
        if let Ok(existing) = exif_service::read_exif(output) {
            merge_missing_exif(&mut info, &existing);
        }
    }

    exif_service::build_exif_app1_payload(&info)
}

/// 只补齐 `info` 中为 `None` 的字段，不覆盖源图已有的值。
fn merge_missing_exif(
    info: &mut crate::models::exif::ExifInfo,
    fallback: &crate::models::exif::ExifInfo,
) {
    macro_rules! fill {
        ($field:ident) => {
            if info.$field.is_none() {
                info.$field = fallback.$field.clone();
            }
        };
    }

    fill!(datetime);
    fill!(gps_latitude);
    fill!(gps_longitude);
    fill!(camera_make);
    fill!(camera_model);
    fill!(lens_model);
    fill!(focal_length);
    fill!(shutter_speed);
    fill!(aperture);
    fill!(iso);
    fill!(rating);
}

// ── 解码 ──────────────────────────────────────────────────────────────────────

/// 用与缩略图一致的链路解码（image crate + EXIF 方向），再转成 8 位 RGB。
/// 带透明通道的图按 gainmap CLI 的 white matte 规则合成到白底。
fn decode_to_rgb8(path: &Path) -> Result<RgbImage, AppError> {
    let bytes = std::fs::read(path)?;
    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut decoder = reader.into_decoder()?;
    // BMP 等格式可能不提供 orientation，失败时按不旋转处理。
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);

    if !img.color().has_alpha() {
        return Ok(img.into_rgb8());
    }

    let rgba = img.into_rgba8();
    let mut rgb = RgbImage::new(rgba.width(), rgba.height());
    for (dst, src) in rgb.pixels_mut().zip(rgba.pixels()) {
        let alpha = src[3] as f64 / 255.0;
        for channel in 0..3 {
            let value = src[channel] as f64 * alpha + 255.0 * (1.0 - alpha);
            dst[channel] = value.round().clamp(0.0, 255.0) as u8;
        }
    }
    Ok(rgb)
}

// ── 增益图 ────────────────────────────────────────────────────────────────────

/// 按 keep-base highlight-selective 模型生成增益图（RGB，逐像素）。
/// 返回 (增益图字节, max_log2)，max_log2 是写入 XMP 的 GainMapMax / HDRCapacityMax。
fn build_gain_map(sdr: &RgbImage, headroom: f64) -> (Vec<u8>, f64) {
    let headroom = headroom.max(1.0001);
    let max_log2 = headroom.log2();
    let inv_log_range = 1.0 / max_log2;

    let src = sdr.as_raw();
    let pixel_count = (sdr.width() as usize) * (sdr.height() as usize);
    let mut gain = vec![0u8; pixel_count * 3];

    for index in 0..pixel_count {
        let offset = index * 3;
        let r = srgb_to_linear(src[offset] as f64 / 255.0);
        let g = srgb_to_linear(src[offset + 1] as f64 / 255.0);
        let b = srgb_to_linear(src[offset + 2] as f64 / 255.0);

        // highlight-selective：亮度越高增益越大（只抬亮高光）。
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let smooth = smoothstep(HIGHLIGHT_EDGE_LOW, HIGHLIGHT_EDGE_HIGH, luma);
        let weight = smooth * smooth;
        let gain_factor = 1.0 + weight * (headroom - 1.0);

        let channels = [r, g, b];
        for (channel, &sdr_linear) in channels.iter().enumerate() {
            let hdr_linear = sdr_linear * gain_factor;
            let ratio = (hdr_linear + OFFSET_HDR) / (sdr_linear + OFFSET_SDR);
            let encoded = ((ratio.log2() - 0.0) * inv_log_range).clamp(0.0, 1.0);
            gain[offset + channel] = (255.0 * encoded).round().clamp(0.0, 255.0) as u8;
        }
    }

    (gain, max_log2)
}

fn srgb_to_linear(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn smoothstep(edge0: f64, edge1: f64, value: f64) -> f64 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ── JPEG 编码 ─────────────────────────────────────────────────────────────────

fn encode_jpeg(pixels: &[u8], width: u32, height: u32, quality: u8) -> Result<Vec<u8>, AppError> {
    let mut buffer = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, quality);
    encoder.encode(pixels, width, height, ExtendedColorType::Rgb8)?;
    Ok(buffer)
}

// ── Ultra HDR 容器组装 ────────────────────────────────────────────────────────

/// 把 SDR 基础图 JPEG 与增益图 JPEG 拼成 Ultra HDR JPEG：
/// SOI + [EXIF APP1] + XMP APP1 + MPF APP2 + 基础图剩余段 +
/// SOI + XMP APP1 + 增益图剩余段。
fn assemble_ultra_hdr_jpeg(
    sdr_jpeg: &[u8],
    gain_jpeg: &[u8],
    max_log2: f64,
    exif_payload: Option<&[u8]>,
) -> Result<Vec<u8>, AppError> {
    if sdr_jpeg.len() < 2 || gain_jpeg.len() < 2 {
        return Err(AppError::General("JPEG 编码结果为空".to_string()));
    }
    if sdr_jpeg[0..2] != [0xFF, MARKER_SOI] || gain_jpeg[0..2] != [0xFF, MARKER_SOI] {
        return Err(AppError::General("JPEG 编码结果缺少 SOI".to_string()));
    }

    let xmp_secondary = generate_xmp_secondary(max_log2);
    let xmp_secondary_payload = xmp_payload(&xmp_secondary);
    let icc_payload = build_icc_app2_payload();
    let icc_segment_len = 4 + icc_payload.len();
    // 第二张图总长度 = SOI + XMP APP1 + ICC APP2 + 增益图（去掉 SOI）。
    let secondary_image_size =
        2 + (4 + xmp_secondary_payload.len()) + icc_segment_len + (gain_jpeg.len() - 2);

    let xmp_primary = generate_xmp_primary(secondary_image_size);
    let xmp_primary_payload = xmp_payload(&xmp_primary);

    let exif_segment_len = exif_payload.map(|payload| 4 + payload.len()).unwrap_or(0);
    let xmp_primary_segment_len = 4 + xmp_primary_payload.len();
    // MPF APP2 段在文件中的起始位置（0xFF 字节的位置）。
    // 与 gainmap CLI 一致：EXIF → XMP → MPF → ICC → 基础图。
    let mpf_start = 2 + exif_segment_len + xmp_primary_segment_len;
    let mpf_segment_len = 4 + MPF_SIZE;

    let primary_image_size = mpf_start + mpf_segment_len + icc_segment_len + (sdr_jpeg.len() - 2);
    // CIPA DC-007：MP Entry 的偏移基准是 MP Endian 字段，即 APP2 段起点 + 8。
    let secondary_offset = primary_image_size
        .checked_sub(mpf_start + 8)
        .ok_or_else(|| AppError::General("MPF 偏移计算失败".to_string()))?;

    let mpf = build_mpf(
        primary_image_size as u32,
        0,
        secondary_image_size as u32,
        secondary_offset as u32,
    );

    let mut output = Vec::with_capacity(primary_image_size + secondary_image_size);
    output.extend_from_slice(&[0xFF, MARKER_SOI]);
    if let Some(payload) = exif_payload {
        push_segment(&mut output, MARKER_APP1, payload);
    }
    push_segment(&mut output, MARKER_APP1, &xmp_primary_payload);
    push_segment(&mut output, MARKER_APP2, &mpf);
    push_segment(&mut output, MARKER_APP2, &icc_payload);
    output.extend_from_slice(&sdr_jpeg[2..]);

    output.extend_from_slice(&[0xFF, MARKER_SOI]);
    push_segment(&mut output, MARKER_APP1, &xmp_secondary_payload);
    push_segment(&mut output, MARKER_APP2, &icc_payload);
    output.extend_from_slice(&gain_jpeg[2..]);

    debug_assert_eq!(output.len(), primary_image_size + secondary_image_size);
    Ok(output)
}

/// 把 sRGB ICC profile 包装成单个 JPEG APP2 段负载（未分块，profile 小于一个 chunk）。
fn build_icc_app2_payload() -> Vec<u8> {
    let mut payload = Vec::with_capacity(14 + SRGB_ICC_PROFILE.len());
    payload.extend_from_slice(b"ICC_PROFILE\0");
    payload.push(1); // chunk sequence number
    payload.push(1); // chunk count
    payload.extend_from_slice(SRGB_ICC_PROFILE);
    payload
}

fn push_segment(output: &mut Vec<u8>, marker: u8, payload: &[u8]) {
    let length = (payload.len() + 2) as u16;
    output.push(0xFF);
    output.push(marker);
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(payload);
}

fn xmp_payload(xml: &str) -> Vec<u8> {
    let mut payload = Vec::with_capacity(XMP_NAMESPACE.len() + xml.len());
    payload.extend_from_slice(XMP_NAMESPACE);
    payload.extend_from_slice(xml.as_bytes());
    payload
}

/// 主图 XMP：Container Directory + hdrgm:Version，声明第二张图是增益图。
fn generate_xmp_primary(secondary_image_size: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines
        .push(r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core 5.1.2">"#.to_string());
    lines
        .push(r#"  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">"#.to_string());
    lines.push(r#"    <rdf:Description"#.to_string());
    lines.push(r#"      xmlns:Container="http://ns.google.com/photos/1.0/container/""#.to_string());
    lines.push(r#"      xmlns:Item="http://ns.google.com/photos/1.0/container/item/""#.to_string());
    lines.push(r#"      xmlns:hdrgm="http://ns.adobe.com/hdr-gain-map/1.0/""#.to_string());
    lines.push(r#"      hdrgm:Version="1.0""#.to_string());
    lines.push(r#"      rdf:about="">"#.to_string());
    lines.push(r#"      <Container:Directory>"#.to_string());
    lines.push(r#"        <rdf:Seq>"#.to_string());
    lines.push(r#"          <rdf:li rdf:parseType="Resource">"#.to_string());
    lines.push(r#"            <Container:Item"#.to_string());
    lines.push(r#"              Item:Semantic="Primary""#.to_string());
    lines.push(r#"              Item:Mime="image/jpeg"/>"#.to_string());
    lines.push(r#"          </rdf:li>"#.to_string());
    lines.push(r#"          <rdf:li rdf:parseType="Resource">"#.to_string());
    lines.push(r#"            <Container:Item"#.to_string());
    lines.push(r#"              Item:Semantic="GainMap""#.to_string());
    lines.push(r#"              Item:Mime="image/jpeg""#.to_string());
    lines.push(format!(
        r#"              Item:Length="{}"/>"#,
        secondary_image_size
    ));
    lines.push(r#"          </rdf:li>"#.to_string());
    lines.push(r#"        </rdf:Seq>"#.to_string());
    lines.push(r#"      </Container:Directory>"#.to_string());
    lines.push(r#"    </rdf:Description>"#.to_string());
    lines.push(r#"  </rdf:RDF>"#.to_string());
    lines.push(r#"</x:xmpmeta>"#.to_string());
    lines.join("\n")
}

/// 增益图 XMP：完整的 hdrgm 重建参数。
fn generate_xmp_secondary(max_log2: f64) -> String {
    let max_value = format_number(max_log2);
    let mut lines: Vec<String> = Vec::new();
    lines
        .push(r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core 5.1.2">"#.to_string());
    lines
        .push(r#"  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">"#.to_string());
    lines.push(r#"    <rdf:Description"#.to_string());
    lines.push(r#"      xmlns:hdrgm="http://ns.adobe.com/hdr-gain-map/1.0/""#.to_string());
    lines.push(r#"      hdrgm:Version="1.0""#.to_string());
    lines.push(r#"      hdrgm:GainMapMin="0""#.to_string());
    lines.push(format!(r#"      hdrgm:GainMapMax="{}""#, max_value));
    lines.push(r#"      hdrgm:Gamma="1""#.to_string());
    lines.push(r#"      hdrgm:OffsetSDR="0.015625""#.to_string());
    lines.push(r#"      hdrgm:OffsetHDR="0.015625""#.to_string());
    lines.push(r#"      hdrgm:HDRCapacityMin="0""#.to_string());
    lines.push(format!(r#"      hdrgm:HDRCapacityMax="{}""#, max_value));
    lines.push(r#"      hdrgm:BaseRenditionIsHDR="False""#.to_string());
    lines.push(r#"      rdf:about=""/>"#.to_string());
    lines.push(r#"  </rdf:RDF>"#.to_string());
    lines.push(r#"</x:xmpmeta>"#.to_string());
    lines.join("\n")
}

/// JS 与 Rust 都使用最短往返浮点格式化，XMP 中的数值直接按 f64 输出即可。
fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    format!("{}", value)
}

// ── MPF ───────────────────────────────────────────────────────────────────────

fn build_mpf(
    primary_image_size: u32,
    primary_image_offset: u32,
    secondary_image_size: u32,
    secondary_image_offset: u32,
) -> Vec<u8> {
    let mut buffer = vec![0u8; MPF_SIZE];
    buffer[0..4].copy_from_slice(b"MPF\0");
    // 小端序，与 gainmap CLI / libultrahdr 参考文件一致。
    buffer[4..6].copy_from_slice(b"II");
    put_u16_le(&mut buffer, 6, 42);
    put_u32_le(&mut buffer, 8, 8); // IFD0 相对 TIFF 头偏移

    put_u16_le(&mut buffer, 12, 3); // tag 数量

    // Tag 1: MPFVersion (UNDEFINED, 4 bytes, "0100")
    put_u16_le(&mut buffer, 14, 0xB000);
    put_u16_le(&mut buffer, 16, 7);
    put_u32_le(&mut buffer, 18, 4);
    buffer[22..26].copy_from_slice(b"0100");

    // Tag 2: NumberOfImages (ULONG, 1, 2)
    put_u16_le(&mut buffer, 26, 0xB001);
    put_u16_le(&mut buffer, 28, 4);
    put_u32_le(&mut buffer, 30, 1);
    put_u32_le(&mut buffer, 34, 2);

    // Tag 3: MPEntry (UNDEFINED, 32 bytes, 偏移 50)
    put_u16_le(&mut buffer, 38, 0xB002);
    put_u16_le(&mut buffer, 40, 7);
    put_u32_le(&mut buffer, 42, 32);
    put_u32_le(&mut buffer, 46, 50);

    put_u32_le(&mut buffer, 50, 0); // 下一个 IFD

    // 主图 MP Entry：JPEG + Baseline MP Primary Image
    put_u32_le(&mut buffer, 54, 0x0003_0000);
    put_u32_le(&mut buffer, 58, primary_image_size);
    put_u32_le(&mut buffer, 62, primary_image_offset);
    // 附加图 MP Entry：JPEG
    put_u32_le(&mut buffer, 70, 0x0000_0000);
    put_u32_le(&mut buffer, 74, secondary_image_size);
    put_u32_le(&mut buffer, 78, secondary_image_offset);

    buffer
}

fn put_u16_le(buffer: &mut [u8], offset: usize, value: u16) {
    buffer[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32_le(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

// ── 写入 ─────────────────────────────────────────────────────────────────────

/// 先写临时文件再重命名，避免覆盖过程中断留下半个 JPG。
/// 临时文件名带进程号与自增序号，避免同一批次里两张源图映射到同一个输出时互相覆盖。
fn atomic_write(path: &Path, data: &[u8]) -> Result<(), AppError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = path.with_extension(format!("myphoto-{}-{}.tmp", std::process::id(), sequence));
    std::fs::write(&tmp, data)?;
    if let Err(error) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(error.into());
    }
    Ok(())
}

// ── 测试 ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgb};

    fn synthetic_image() -> RgbImage {
        let mut img = RgbImage::new(48, 32);
        for y in 0..32u32 {
            for x in 0..48u32 {
                let gradient = ((x + y) * 255 / 78).min(255) as u8;
                img.put_pixel(x, y, Rgb([gradient, gradient / 2 + 40, 255 - gradient / 2]));
            }
        }
        img
    }

    fn find_mpf(data: &[u8]) -> usize {
        assert_eq!(data[0..2], [0xFF, MARKER_SOI]);
        let mut pos = 2;
        loop {
            assert_eq!(data[pos], 0xFF, "expected JPEG marker");
            let marker = data[pos + 1];
            assert!(
                marker != 0xDA && marker != 0xD9,
                "MPF segment not found before SOS/EOI"
            );
            let length = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            if marker == MARKER_APP2 && &data[pos + 4..pos + 8] == b"MPF\0" {
                return pos;
            }
            pos += 2 + length;
        }
    }

    fn read_u16_le(data: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes([data[offset], data[offset + 1]])
    }

    fn read_u32_le(data: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ])
    }

    /// 从 MPF 中读出第二张图相对 MP Endian 字段的偏移与长度。
    fn secondary_image_location(data: &[u8]) -> (usize, usize) {
        let mpf_start = find_mpf(data);
        let tiff = mpf_start + 8;
        let ifd_offset = read_u32_le(data, tiff + 4) as usize;
        let tag_count = read_u16_le(data, tiff + ifd_offset) as usize;
        let mut entries_offset = 0usize;
        for index in 0..tag_count {
            let entry = tiff + ifd_offset + 2 + index * 12;
            if read_u16_le(data, entry) == 0xB002 {
                entries_offset = read_u32_le(data, entry + 8) as usize;
            }
        }
        assert!(entries_offset > 0, "MPEntry tag missing");
        let second_entry = tiff + entries_offset + 16;
        let size = read_u32_le(data, second_entry + 4) as usize;
        let offset = read_u32_le(data, second_entry + 8) as usize;
        (tiff + offset, size)
    }

    #[test]
    fn test_convert_synthetic_tiff_to_ultra_hdr() {
        let dir = std::env::temp_dir().join("myphoto_hdr_test");
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("synthetic_hdr.tif");
        synthetic_image()
            .save_with_format(&input, ImageFormat::Tiff)
            .unwrap();

        let outcome = convert_file_to_hdr(&input, None).expect("conversion should succeed");
        assert_eq!(outcome.output_path, dir.join("synthetic_hdr.jpg"));
        assert_eq!((outcome.width, outcome.height), (48, 32));

        let bytes = std::fs::read(&outcome.output_path).unwrap();
        assert_eq!(bytes[0..2], [0xFF, MARKER_SOI]);

        // 主图 XMP 必须带 hdrgm:Version（应用据此显示 HDR 角标）。
        let header_len = bytes.len().min(8192);
        let header = String::from_utf8_lossy(&bytes[..header_len]);
        assert!(
            header.contains("hdrgm:Version=\"1.0\""),
            "primary XMP missing hdrgm version"
        );
        assert!(
            header.contains("Item:Semantic=\"GainMap\""),
            "primary XMP missing gain map item"
        );

        let (secondary_start, secondary_size) = secondary_image_location(&bytes);
        assert_eq!(secondary_start + secondary_size, bytes.len());
        assert_eq!(
            &bytes[secondary_start..secondary_start + 2],
            &[0xFF, MARKER_SOI]
        );

        // MPF 的主图长度字段应正好指向第二张图起点，XMP 的 Item:Length 应等于第二张图长度。
        let mpf_start = find_mpf(&bytes);
        let tiff = mpf_start + 8;
        // 相对 TIFF 头：IFD0 在 8，条目在 50；主图 Entry 的 size 在 54、offset 在 58。
        assert_eq!(
            read_u32_le(&bytes, tiff + 50 + 4) as usize,
            secondary_start,
            "MPF 主图长度字段不正确"
        );
        assert_eq!(
            read_u32_le(&bytes, tiff + 50 + 20) as usize,
            secondary_size,
            "MPF 增益图长度字段不正确"
        );
        assert!(
            header.contains(&format!("Item:Length=\"{}\"", secondary_size)),
            "XMP 增益图长度与 MPF 不一致"
        );

        let secondary_xmp = String::from_utf8_lossy(
            &bytes[secondary_start..(secondary_start + 2048).min(bytes.len())],
        );
        assert!(
            secondary_xmp.contains("hdrgm:GainMapMax"),
            "secondary XMP missing params"
        );

        // 基础图与增益图都应带 sRGB ICC APP2（与 gainmap CLI 输出一致）。
        for (label, start) in [("primary", 0usize), ("secondary", secondary_start)] {
            let icc_segment =
                find_icc(&bytes, start).unwrap_or_else(|| panic!("{label} ICC segment missing"));
            let payload = icc_segment + 4;
            assert_eq!(bytes[payload + 12], 1, "{label} ICC chunk index");
            assert_eq!(bytes[payload + 13], 1, "{label} ICC chunk count");
            let profile = payload + 14;
            assert_eq!(
                &bytes[profile + 36..profile + 40],
                b"acsp",
                "{label} ICC invalid"
            );
            assert_eq!(
                &bytes[profile..profile + 4],
                &(SRGB_ICC_PROFILE.len() as u32).to_be_bytes(),
                "{label} ICC size mismatch"
            );
        }

        // 基础图可解码且与源图基本一致（JPEG q92）。
        let base = image::load_from_memory(&bytes[..secondary_start])
            .expect("primary JPEG should decode")
            .to_rgb8();
        assert_eq!(base.dimensions(), (48, 32));
        let source = synthetic_image();
        let mut total_diff = 0u64;
        for (a, b) in base.pixels().zip(source.pixels()) {
            for channel in 0..3 {
                total_diff += (a[channel] as i32 - b[channel] as i32).unsigned_abs() as u64;
            }
        }
        let mean_diff = total_diff as f64 / (48.0 * 32.0 * 3.0);
        assert!(mean_diff < 8.0, "SDR base drifted too much: {mean_diff}");

        // 增益图可解码，且按 XMP 参数重建的 HDR 与 highlight-selective 模型一致。
        let gain = image::load_from_memory(&bytes[secondary_start..])
            .expect("gain map JPEG should decode")
            .to_rgb8();
        let max_log2 = DEFAULT_HEADROOM.log2();
        let mut max_error = 0.0f64;
        let mut total_error = 0.0f64;
        let mut samples = 0u64;
        for (base_px, gain_px) in base.pixels().zip(gain.pixels()) {
            let sdr = base_px.0.map(|v| srgb_to_linear(v as f64 / 255.0));
            let luma = 0.2126 * sdr[0] + 0.7152 * sdr[1] + 0.0722 * sdr[2];
            let smooth = smoothstep(HIGHLIGHT_EDGE_LOW, HIGHLIGHT_EDGE_HIGH, luma);
            let gain_factor = 1.0 + smooth * smooth * (DEFAULT_HEADROOM - 1.0);
            for channel in 0..3 {
                let gain_log2 = gain_px[channel] as f64 / 255.0 * max_log2;
                let reconstructed = (sdr[channel] + OFFSET_SDR) * 2f64.powf(gain_log2) - OFFSET_HDR;
                let expected = sdr[channel] * gain_factor;
                let error = (reconstructed - expected).abs();
                max_error = max_error.max(error);
                total_error += error;
                samples += 1;
            }
        }
        let mean_error = total_error / samples as f64;
        assert!(
            max_error < 0.2,
            "gain map max reconstruction error too large: {max_error}"
        );
        assert!(
            mean_error < 0.02,
            "gain map mean reconstruction error too large: {mean_error}"
        );
    }

    #[test]
    fn test_exif_fields_are_copied_to_output() {
        use crate::models::exif::ExifWriteRequest;

        let dir = std::env::temp_dir().join("myphoto_hdr_exif_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // TIF → 写入评级 / 相机型号 → 转换后应在新 JPG 中读到同样的值。
        let input = dir.join("rated.tif");
        synthetic_image()
            .save_with_format(&input, ImageFormat::Tiff)
            .unwrap();
        crate::services::exif_service::write_exif_fields(
            &[input.to_string_lossy().to_string()],
            &ExifWriteRequest {
                rating: Some(4),
                camera_model: Some("HDR Test Camera".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let outcome = convert_file_to_hdr(&input, None).unwrap();
        let info = crate::services::exif_service::read_exif(&outcome.output_path).unwrap();
        assert_eq!(info.rating, Some(4), "评级未复制到 HDR JPG");
        assert_eq!(info.camera_model.as_deref(), Some("HDR Test Camera"));

        // 之后再用「修改 EXIF」写入评级，不能破坏 Ultra HDR 结构。
        crate::services::exif_service::write_exif_fields(
            &[outcome.output_path.to_string_lossy().to_string()],
            &ExifWriteRequest {
                rating: Some(3),
                ..Default::default()
            },
        )
        .unwrap();
        let bytes = std::fs::read(&outcome.output_path).unwrap();
        let (secondary_start, secondary_size) = secondary_image_location(&bytes);
        assert_eq!(secondary_start + secondary_size, bytes.len());
        assert_eq!(
            &bytes[secondary_start..secondary_start + 2],
            &[0xFF, MARKER_SOI],
            "修改 EXIF 后增益图偏移失效"
        );
        let header = String::from_utf8_lossy(&bytes[..4096.min(bytes.len())]);
        assert!(header.contains("hdrgm:Version"), "修改 EXIF 后丢失 HDR XMP");
        assert!(
            image::load_from_memory(&bytes[secondary_start..]).is_ok(),
            "修改 EXIF 后增益图无法解码"
        );

        // BMP 没有 EXIF：覆盖已有同名 JPG 时应保留旧 JPG 的评级。
        let bmp_input = dir.join("rated.bmp");
        synthetic_image()
            .save_with_format(&bmp_input, ImageFormat::Bmp)
            .unwrap();
        let existing_jpg = dir.join("rated.jpg");
        let mut fallback = std::fs::File::create(&existing_jpg).unwrap();
        synthetic_image()
            .write_to(&mut fallback, ImageFormat::Jpeg)
            .unwrap();
        crate::services::exif_service::write_exif_fields(
            &[existing_jpg.to_string_lossy().to_string()],
            &ExifWriteRequest {
                rating: Some(5),
                ..Default::default()
            },
        )
        .unwrap();

        let outcome = convert_file_to_hdr(&bmp_input, None).unwrap();
        let info = crate::services::exif_service::read_exif(&outcome.output_path).unwrap();
        assert_eq!(info.rating, Some(5), "覆盖已有 JPG 时未保留旧评级");
    }

    /// 从一个 JPEG 起始位置扫描到 SOS，找到 ICC APP2 段则返回段起点。
    fn find_icc(data: &[u8], start: usize) -> Option<usize> {
        if data.get(start..start + 2) != Some(&[0xFF, MARKER_SOI]) {
            return None;
        }
        let mut pos = start + 2;
        loop {
            if data.get(pos) != Some(&0xFF) {
                return None;
            }
            let marker = *data.get(pos + 1)?;
            if marker == 0xDA || marker == 0xD9 {
                return None;
            }
            let length = u16::from_be_bytes([*data.get(pos + 2)?, *data.get(pos + 3)?]) as usize;
            if marker == MARKER_APP2 && data.get(pos + 4..pos + 16) == Some(b"ICC_PROFILE\0") {
                return Some(pos);
            }
            pos += 2 + length;
        }
    }

    #[test]
    fn test_output_path_replaces_extension() {
        assert_eq!(
            output_path_for(Path::new("C:/photos/IMG_0001.TIF")),
            Some(PathBuf::from("C:/photos/IMG_0001.jpg"))
        );
        assert_eq!(
            output_path_for(Path::new("C:/photos/scan.v2.BMP")),
            Some(PathBuf::from("C:/photos/scan.v2.jpg"))
        );
    }

    #[test]
    fn test_supported_extensions() {
        assert!(is_convertible_file(Path::new("a.tif")));
        assert!(is_convertible_file(Path::new("a.TIFF")));
        assert!(is_convertible_file(Path::new("a.bmp")));
        assert!(!is_convertible_file(Path::new("a.jpg")));
        assert!(!is_convertible_file(Path::new("a.raf")));
    }

    /// 用 test_data 中的真实 TIF / BMP 验证解码、EXIF 复制与应用侧 HDR 检测。
    /// 手动运行：cargo test --manifest-path src-tauri/Cargo.toml real_test_data -- --ignored --nocapture
    #[test]
    #[ignore = "需要 test_data 中的真实大图，手动运行"]
    fn test_convert_real_test_data() {
        use std::path::PathBuf;

        let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        let work_dir = std::env::temp_dir().join("myphoto_hdr_real_test");
        let _ = std::fs::remove_dir_all(&work_dir);
        std::fs::create_dir_all(&work_dir).unwrap();

        for name in ["000226830002.tif", "000010.BMP"] {
            let source = source_dir.join(name);
            if !source.exists() {
                println!("跳过：{} 不存在", source.display());
                continue;
            }
            let input = work_dir.join(name);
            std::fs::copy(&source, &input).unwrap();

            let started = std::time::Instant::now();
            let outcome = convert_file_to_hdr(&input, None).expect("真实图片转换失败");
            println!(
                "{} -> {} ({}x{}, 耗时 {:?})",
                name,
                outcome.output_path.display(),
                outcome.width,
                outcome.height,
                started.elapsed()
            );

            let bytes = std::fs::read(&outcome.output_path).unwrap();
            let head = String::from_utf8_lossy(&bytes[..4096.min(bytes.len())]).to_string();
            assert!(head.contains("hdrgm:Version"), "输出缺少 HDR XMP");

            // 基础图（SDR 层）应与解码后的源图基本一致。
            let (secondary_start, _) = secondary_image_location(&bytes);
            let base = image::load_from_memory(&bytes[..secondary_start])
                .expect("基础图 JPEG 无法解码")
                .to_rgb8();
            let source_rgb = decode_to_rgb8(&input).expect("源图无法解码");
            assert_eq!(base.dimensions(), source_rgb.dimensions());
            let mut total_diff = 0u64;
            for (a, b) in base.pixels().zip(source_rgb.pixels()) {
                for channel in 0..3 {
                    total_diff += (a[channel] as i32 - b[channel] as i32).unsigned_abs() as u64;
                }
            }
            let mean_diff = total_diff as f64 / (base.width() as f64 * base.height() as f64 * 3.0);
            println!("  SDR 平均通道误差: {mean_diff:.2}");
            assert!(mean_diff < 8.0, "SDR 基础图与原图差异过大: {mean_diff}");

            // 应用侧的 list_images 应把新 JPG 识别为 HDR 并挂回同名图片组。
            let groups = crate::services::image_processor::list_images(&work_dir).unwrap();
            let base_name = outcome
                .output_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap();
            let group = groups
                .iter()
                .find(|group| group.base_name == base_name)
                .expect("转换后的图片组不存在");
            assert!(group.is_hdr, "list_images 未识别出 HDR");
            let jpg_path = group.jpg_path.clone().expect("图片组缺少 jpgPath");
            assert_eq!(
                jpg_path,
                outcome.output_path.to_string_lossy().to_string(),
                "图片组 jpgPath 与转换输出不一致"
            );

            // 源 TIF 的 EXIF 应被复制到输出 JPG。
            let info = exif_service::read_exif(&outcome.output_path).unwrap();
            println!(
                "  copied exif: datetime={:?} model={:?} lens={:?} iso={:?}",
                info.datetime, info.camera_model, info.lens_model, info.iso
            );
            assert!(
                info.image_width == Some(outcome.width)
                    && info.image_height == Some(outcome.height),
                "输出 JPG 尺寸信息不正确"
            );
        }
    }
}
