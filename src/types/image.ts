export interface ImageGroup {
  baseName: string
  jpgPath: string | null
  rawPath: string | null
  fileCount: number
  modifiedTime: string | null
}

export type SortField = 'date' | 'rating'
export type SortOrder = 'asc' | 'desc'

export interface SortOptions {
  field: SortField
  order: SortOrder
}
