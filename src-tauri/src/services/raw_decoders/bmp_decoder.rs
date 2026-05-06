use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};

/// BMP文件头结构 (14字节)
#[derive(Debug)]
pub struct BmpFileHeader {
    /// 文件类型标识 ("BM")
    pub signature: [u8; 2],
    /// 文件总大小（字节）
    pub file_size: u32,
    /// 保留字段1
    pub reserved1: u16,
    /// 保留字段2
    pub reserved2: u16,
    /// 像素数据起始偏移量
    pub data_offset: u32,
}

impl BmpFileHeader {
    /// 从BMP文件中解析文件头
    pub fn parse<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;

        // 读取签名 (2 bytes)
        let mut signature = [0u8; 2];
        file.read_exact(&mut signature)?;

        // 验证BMP签名
        if &signature != b"BM" {
            return Err("无效的BMP文件签名，期望'BM'".into());
        }

        // 读取文件头其余字段 (Little Endian)
        let file_size = file.read_u32::<LittleEndian>()?;
        let reserved1 = file.read_u16::<LittleEndian>()?;
        let reserved2 = file.read_u16::<LittleEndian>()?;
        let data_offset = file.read_u32::<LittleEndian>()?;

        Ok(BmpFileHeader {
            signature,
            file_size,
            reserved1,
            reserved2,
            data_offset,
        })
    }
}

/// BMP信息头结构 (BITMAPINFOHEADER - 40字节)
#[derive(Debug)]
pub struct BmpInfoHeader {
    /// 信息头大小（字节）
    pub header_size: u32,
    /// 图像宽度（像素）
    pub width: i32,
    /// 图像高度（像素，正值=底部向上，负值=顶部向下）
    pub height: i32,
    /// 颜色平面数（必须为1）
    pub planes: u16,
    /// 每像素位数 (1, 4, 8, 16, 24, 32)
    pub bits_per_pixel: u16,
    /// 压缩方式 (0=无压缩, 1=RLE8, 2=RLE4, 3=Bitfields)
    pub compression: u32,
    /// 图像数据大小（字节，无压缩时可为0）
    pub image_size: u32,
    /// 水平分辨率（像素/米）
    pub x_pixels_per_meter: i32,
    /// 垂直分辨率（像素/米）
    pub y_pixels_per_meter: i32,
    /// 颜色表中使用的颜色数
    pub colors_used: u32,
    /// 重要颜色数（0=全部重要）
    pub colors_important: u32,
}

impl BmpInfoHeader {
    /// 从BMP文件中解析信息头
    pub fn parse<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        // 跳过文件头 (14字节)
        file.seek(SeekFrom::Start(14))?;

        let header_size = file.read_u32::<LittleEndian>()?;
        let width = file.read_i32::<LittleEndian>()?;
        let height = file.read_i32::<LittleEndian>()?;
        let planes = file.read_u16::<LittleEndian>()?;
        let bits_per_pixel = file.read_u16::<LittleEndian>()?;
        let compression = file.read_u32::<LittleEndian>()?;
        let image_size = file.read_u32::<LittleEndian>()?;
        let x_pixels_per_meter = file.read_i32::<LittleEndian>()?;
        let y_pixels_per_meter = file.read_i32::<LittleEndian>()?;
        let colors_used = file.read_u32::<LittleEndian>()?;
        let colors_important = file.read_u32::<LittleEndian>()?;

        Ok(BmpInfoHeader {
            header_size,
            width,
            height,
            planes,
            bits_per_pixel,
            compression,
            image_size,
            x_pixels_per_meter,
            y_pixels_per_meter,
            colors_used,
            colors_important,
        })
    }
}

/// 颜色表条目
#[derive(Debug, Clone)]
pub struct ColorTableEntry {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub reserved: u8,
}

/// BMP像素数据结构
#[derive(Debug)]
pub struct PixelData {
    /// 原始像素数据
    pub data: Vec<u8>,
    /// 颜色表（索引色模式使用）
    pub color_table: Vec<ColorTableEntry>,
}

impl PixelData {
    /// 从BMP文件中提取像素数据
    pub fn extract<P: AsRef<Path>>(
        filename: P,
        file_header: &BmpFileHeader,
        info_header: &BmpInfoHeader,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(&filename)?;

        // 读取颜色表（如果存在）
        let mut color_table = Vec::new();
        if info_header.bits_per_pixel <= 8 {
            let color_count = if info_header.colors_used > 0 {
                info_header.colors_used
            } else {
                1u32 << info_header.bits_per_pixel
            };

            file.seek(SeekFrom::Start(14 + info_header.header_size as u64))?;
            for _ in 0..color_count {
                let blue = file.read_u8()?;
                let green = file.read_u8()?;
                let red = file.read_u8()?;
                let reserved = file.read_u8()?;
                color_table.push(ColorTableEntry {
                    blue,
                    green,
                    red,
                    reserved,
                });
            }
        }

        // 读取像素数据
        file.seek(SeekFrom::Start(file_header.data_offset as u64))?;

        let data_size = if info_header.image_size > 0 {
            info_header.image_size as usize
        } else {
            // 计算数据大小：每行需要4字节对齐
            let row_size = ((info_header.bits_per_pixel as i32 * info_header.width.abs() + 31) / 32 * 4) as usize;
            row_size * info_header.height.unsigned_abs() as usize
        };

        let mut data = vec![0u8; data_size];
        file.read_exact(&mut data)?;

        Ok(PixelData { data, color_table })
    }
}

/// BMP文件解码器主结构
#[derive(Debug)]
pub struct BmpDecoder {
    /// 文件路径
    pub filename: String,
    /// 文件头
    pub file_header: BmpFileHeader,
    /// 信息头
    pub info_header: BmpInfoHeader,
    /// 像素数据
    pub pixel_data: PixelData,
}

impl BmpDecoder {
    /// 创建新的BMP解码器实例
    pub fn new<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let filename_str = filename.as_ref().to_string_lossy().to_string();

        // 解析文件头
        let file_header = BmpFileHeader::parse(&filename)?;

        // 解析信息头
        let info_header = BmpInfoHeader::parse(&filename)?;

        // 提取像素数据
        let pixel_data = PixelData::extract(&filename, &file_header, &info_header)?;

        Ok(BmpDecoder {
            filename: filename_str,
            file_header,
            info_header,
            pixel_data,
        })
    }

    /// 导出原始像素数据到文件
    pub fn export_data<P: AsRef<Path>>(&self, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(output_path, &self.pixel_data.data)?;
        Ok(())
    }

    /// 获取图像信息摘要
    pub fn get_camera_info(&self) -> String {
        format!(
            "BMP {}x{} {}bpp",
            self.info_header.width.abs(),
            self.info_header.height.abs(),
            self.info_header.bits_per_pixel
        )
    }

    /// 获取图像尺寸
    pub fn get_image_dimensions(&self) -> (u32, u32) {
        (
            self.info_header.width.unsigned_abs(),
            self.info_header.height.unsigned_abs(),
        )
    }

    /// 获取像素数据大小（字节）
    pub fn get_data_size(&self) -> usize {
        self.pixel_data.data.len()
    }

    /// 获取文件总大小（字节）
    pub fn get_file_size(&self) -> u32 {
        self.file_header.file_size
    }

    /// 获取每像素位数
    pub fn get_bits_per_pixel(&self) -> u16 {
        self.info_header.bits_per_pixel
    }

    /// 获取压缩方式描述
    pub fn get_compression_info(&self) -> &str {
        match self.info_header.compression {
            0 => "无压缩 (BI_RGB)",
            1 => "RLE 8位 (BI_RLE8)",
            2 => "RLE 4位 (BI_RLE4)",
            3 => "位域 (BI_BITFIELDS)",
            4 => "JPEG (BI_JPEG)",
            5 => "PNG (BI_PNG)",
            _ => "未知压缩方式",
        }
    }

    /// 获取分辨率信息（DPI）
    pub fn get_resolution_dpi(&self) -> (f64, f64) {
        // 像素/米 转换为 DPI (1英寸 = 0.0254米)
        let x_dpi = self.info_header.x_pixels_per_meter as f64 * 0.0254;
        let y_dpi = self.info_header.y_pixels_per_meter as f64 * 0.0254;
        (x_dpi, y_dpi)
    }

    /// 获取颜色表大小
    pub fn get_color_table_size(&self) -> usize {
        self.pixel_data.color_table.len()
    }

    /// 图像方向是否为自底向上
    pub fn is_bottom_up(&self) -> bool {
        self.info_header.height > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_bmp_decoder_basic() {
        let test_file = "test_data/000010.BMP";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = BmpDecoder::new(test_file).expect("无法解码BMP文件");

        // 验证基本信息
        let (width, height) = decoder.get_image_dimensions();
        assert!(width > 0, "图像宽度应该大于0");
        assert!(height > 0, "图像高度应该大于0");
        assert!(decoder.get_data_size() > 0, "像素数据大小应该大于0");
        assert!(decoder.get_file_size() > 0, "文件大小应该大于0");

        println!("图像信息: {}", decoder.get_camera_info());
        println!("图像尺寸: {}x{}", width, height);
        println!("数据大小: {} bytes", decoder.get_data_size());
        println!("文件大小: {} bytes", decoder.get_file_size());
        println!("位深度: {} bpp", decoder.get_bits_per_pixel());
        println!("压缩方式: {}", decoder.get_compression_info());
        println!("自底向上: {}", decoder.is_bottom_up());
    }

    #[test]
    fn test_bmp_header_structure() {
        let test_file = "test_data/000010.BMP";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let file_header = BmpFileHeader::parse(test_file).expect("无法解析BMP文件头");

        // 验证BMP签名
        assert_eq!(&file_header.signature, b"BM", "文件签名应该为'BM'");

        // 验证文件大小合理
        assert!(file_header.file_size > 0, "文件大小应该大于0");

        // 验证数据偏移量合理
        assert!(file_header.data_offset >= 54, "数据偏移量应该至少为54字节");

        let info_header = BmpInfoHeader::parse(test_file).expect("无法解析BMP信息头");

        // 验证信息头大小（至少40字节表示BITMAPINFOHEADER）
        assert!(info_header.header_size >= 40, "信息头大小应该至少为40字节");

        // 验证颜色平面数
        assert_eq!(info_header.planes, 1, "颜色平面数应该为1");

        // 验证位深度
        assert!(
            [1, 4, 8, 16, 24, 32].contains(&info_header.bits_per_pixel),
            "位深度应该是有效值: {}",
            info_header.bits_per_pixel
        );

        println!("BMP签名: {:?}", String::from_utf8_lossy(&file_header.signature));
        println!("文件大小: {} bytes", file_header.file_size);
        println!("数据偏移: {} bytes", file_header.data_offset);
        println!("信息头大小: {} bytes", info_header.header_size);
        println!("图像尺寸: {}x{}", info_header.width, info_header.height);
        println!("位深度: {} bpp", info_header.bits_per_pixel);
    }

    #[test]
    fn test_bmp_data_extraction() {
        let test_file = "test_data/000010.BMP";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = BmpDecoder::new(test_file).expect("无法解码BMP文件");

        // 验证像素数据提取正确性
        let data_size = decoder.get_data_size();
        let (width, height) = decoder.get_image_dimensions();

        assert!(data_size > 0, "像素数据不应为空");

        // 验证数据大小与图像尺寸匹配
        let bits_per_pixel = decoder.get_bits_per_pixel() as u32;
        let row_size = ((bits_per_pixel * width + 31) / 32 * 4) as usize;
        let expected_size = row_size * height as usize;

        if decoder.info_header.compression == 0 {
            assert_eq!(
                data_size, expected_size,
                "无压缩数据大小应该匹配: 实际={}, 期望={}",
                data_size, expected_size
            );
        }

        println!("数据提取验证通过");
        println!("像素数据大小: {} bytes", data_size);
        println!("期望数据大小: {} bytes", expected_size);
    }

    #[test]
    fn test_bmp_export_data() {
        let test_file = "test_data/000010.BMP";
        let output_file = "test_data/bmp_output.raw";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = BmpDecoder::new(test_file).expect("无法解码BMP文件");
        decoder.export_data(output_file).expect("无法导出像素数据");

        // 验证输出文件存在且不为空
        let output_path = PathBuf::from(output_file);
        assert!(output_path.exists(), "输出文件应该存在");
        let metadata = std::fs::metadata(&output_path).expect("无法读取输出文件元数据");
        assert!(metadata.len() > 0, "输出文件不应为空");

        // 清理测试文件
        let _ = std::fs::remove_file(output_file);
    }
}
