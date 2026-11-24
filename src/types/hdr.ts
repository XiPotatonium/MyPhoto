/// HDR 转换计划 / 结果（字段与 Rust 命令的 camelCase 序列化保持一致）。

export interface HdrPlan {
  /// 选中的照片总数
  total: number
  /// 支持转换的 TIF/BMP 数量
  convertible: number
  /// 不支持转换的文件名
  unsupported: string[]
  /// 已存在同名 JPG、会被覆盖的文件名
  overwrite: string[]
}

export interface HdrConvertResult {
  inputPath: string
  outputPath: string | null
  success: boolean
  error: string | null
}
