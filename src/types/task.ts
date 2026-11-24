/// 后台批量任务（HDR 转换 / EXIF 写入）的前端状态与事件类型。

export type TaskKind = 'hdr' | 'exif'

export interface TaskProgress {
  id: string
  kind: TaskKind
  /// 进度浮层标题，例如「转换为 HDR」
  title: string
  processed: number
  total: number
  /// 刚处理完的文件名
  current: string | null
  failed: number
  /// 全部处理完成（progress 浮层稍后自动关闭）
  done: boolean
}

/// Rust 侧 `task-progress` 事件负载（camelCase）。
export interface TaskProgressPayload {
  taskId: string
  kind: TaskKind
  processed: number
  total: number
  current: string | null
  failed: number
}
