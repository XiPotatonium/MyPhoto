export interface ExifInfo {
  datetime: string | null
  gpsLatitude: number | null
  gpsLongitude: number | null
  cameraMake: string | null
  cameraModel: string | null
  lensModel: string | null
  focalLength: number | null
  shutterSpeed: string | null
  aperture: number | null
  iso: number | null
  rating: number | null
  imageWidth: number | null
  imageHeight: number | null
  orientation: number | null
  fileCreatedTime: string | null
}

/// 批量写 EXIF 的单项结果（与 Rust `write_exif_fields_batch` 对应）。
export interface ExifWriteResult {
  filePath: string
  success: boolean
  error: string | null
}
