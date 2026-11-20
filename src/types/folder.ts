import type { ImageGroup } from './image'

/// 0.3.0：文件浏览器直接展示子文件夹时的条目数据。
/// photoCount 是直属图片组数（同名 JPG+RAW 算一张），不递归统计。
export interface FolderItem {
  name: string
  path: string
  photoCount: number
  coverPath: string | null
}

export interface DirectoryContents {
  folders: FolderItem[]
  images: ImageGroup[]
}

/// 网格 / strip 中的统一条目：文件夹恒排在图片之前。
export type BrowserEntry =
  | { kind: 'folder'; key: string; folder: FolderItem }
  | { kind: 'image'; key: string; image: ImageGroup }

export function folderKey(path: string): string {
  return `folder:${path}`
}
