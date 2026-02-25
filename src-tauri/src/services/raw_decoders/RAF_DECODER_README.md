# RAF解码器测试说明

## 概述

RAF解码器是用于解析富士相机RAF原始文件格式的Rust实现，参考了 [RAF2jpg](https://github.com/GCaptainNemo/RAF2jpg) 的Python实现。

## 功能特性

- ✅ 解析RAF文件头部信息
- ✅ 提取相机信息（型号、ID等）
- ✅ 提取嵌入的JPEG预览图（包含EXIF数据）
- ✅ 提取CFA（Color Filter Array）原始传感器数据头部
- ✅ 导出JPEG预览图到文件

## 文件结构

RAF文件格式包含以下主要部分：

1. **文件头部** (0-100字节)
   - 类型标识 (16字节): "FUJIFILMCCD-RAW"
   - 格式版本 (4字节)
   - 相机ID (8字节)
   - 相机型号字符串 (32字节)
   - 各数据块的偏移量和长度

2. **JPEG数据块**
   - 包含EXIF信息
   - 包含预览图
   - 通常是全分辨率的JPEG

3. **CFA头部**
   - 包含多个标签记录
   - 描述原始传感器数据的参数

4. **CFA数据块**
   - 原始传感器数据（本实现暂不解析）

## 运行测试

### 准备测试文件

1. 将一个富士RAF文件（如 `.RAF` 格式）放入 `src-tauri/test_data/` 目录
2. 将文件重命名为 `sample.RAF`

### 运行所有测试

```bash
cd src-tauri
cargo test raf_decoder -- --nocapture
```

### 运行特定测试

```bash
# 测试基本解码功能
cargo test test_raf_decoder_basic -- --nocapture

# 测试JPEG导出
cargo test test_export_jpeg -- --nocapture

# 测试头部解析
cargo test test_raf_header_structure -- --nocapture
```

## 使用示例

```rust
use myphoto_lib::services::raf_decoder::RafDecoder;

// 创建解码器实例
let decoder = RafDecoder::new("path/to/file.RAF")?;

// 获取相机信息
println!("相机: {}", decoder.get_camera_info());

// 获取JPEG大小
println!("JPEG大小: {} bytes", decoder.get_jpeg_size());

// 导出JPEG预览图
decoder.export_jpeg("output.jpg")?;

// 获取CFA记录数
println!("CFA记录数: {}", decoder.get_cfa_record_count());
```

## API文档

### `RafDecoder`

主解码器结构，用于解析RAF文件。

#### 方法

- `new(filename)` - 创建新的解码器实例并解析文件
- `export_jpeg(output_path)` - 导出JPEG预览图到文件
- `get_camera_info()` - 获取相机型号字符串
- `get_jpeg_size()` - 获取JPEG数据大小（字节）
- `get_cfa_record_count()` - 获取CFA记录数量

### `RafHeader`

RAF文件头部信息结构。

#### 方法

- `parse(filename)` - 从文件解析头部信息

### `JpegData`

JPEG数据结构。

#### 方法

- `extract(filename, offset, length)` - 从RAF文件提取JPEG数据
- `save(path)` - 将JPEG数据保存到文件

### `CfaData`

CFA（原始传感器数据）结构。

#### 方法

- `extract(filename, header_offset, header_length, data_offset, data_length)` - 提取CFA数据

## 注意事项

1. 所有偏移量和长度使用**大端序**（Big Endian）编码
2. JPEG数据通常包含完整的EXIF信息
3. CFA数据解包功能尚未实现（DNG转换）
4. 测试需要真实的RAF文件才能通过

## 依赖项

- `byteorder` - 用于大端序数据读取
- Rust标准库的文件I/O

## 参考资料

- [RAF2jpg Python实现](https://github.com/GCaptainNemo/RAF2jpg)
- 富士RAF文件格式规范（非官方）

## 后续改进

- [ ] 实现CFA数据解包
- [ ] 支持DNG格式导出
- [ ] 支持更多富士相机型号
- [ ] 优化内存使用
- [ ] 添加错误恢复机制
