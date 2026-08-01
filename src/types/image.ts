import type { ExifInfo } from './exif'

export interface ImageGroup {
  baseName: string
  jpgPath: string | null
  rawPath: string | null
  fileCount: number
  isHdr: boolean
  modifiedTime: string | null
  fileCreatedTime: string | null
  exifInfo: ExifInfo | null
}

export type SortField = 'name' | 'date' | 'rating'
export type SortOrder = 'asc' | 'desc'

export interface SortOptions {
  field: SortField
  order: SortOrder
}

/// 后端 read_exif_batch / exif-ready 事件的单项结果（字段 snake_case）。
export interface ExifBatchResult {
  session_id: string
  base_name: string
  exif_info: ExifInfo | null
  error: string | null
}
