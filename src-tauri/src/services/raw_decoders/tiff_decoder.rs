use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

/// TIFF字节序类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteOrder {
    /// Intel字节序 (Little Endian) - "II"
    LittleEndian,
    /// Motorola字节序 (Big Endian) - "MM"
    BigEndian,
}

/// TIFF文件头结构
#[derive(Debug)]
pub struct TiffHeader {
    /// 字节序标识
    pub byte_order: ByteOrder,
    /// 版本号 (应为42)
    pub version: u16,
    /// 第一个IFD的偏移量
    pub ifd_offset: u32,
}

impl TiffHeader {
    /// 从TIFF文件中解析头部信息
    pub fn parse<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;

        // 读取字节序标识 (2 bytes)
        let mut order_bytes = [0u8; 2];
        file.read_exact(&mut order_bytes)?;

        let byte_order = match &order_bytes {
            b"II" => ByteOrder::LittleEndian,
            b"MM" => ByteOrder::BigEndian,
            _ => return Err("无效的TIFF字节序标识".into()),
        };

        // 读取版本号和IFD偏移量（根据字节序）
        let (version, ifd_offset) = match byte_order {
            ByteOrder::LittleEndian => {
                let version = file.read_u16::<LittleEndian>()?;
                let ifd_offset = file.read_u32::<LittleEndian>()?;
                (version, ifd_offset)
            }
            ByteOrder::BigEndian => {
                let version = file.read_u16::<BigEndian>()?;
                let ifd_offset = file.read_u32::<BigEndian>()?;
                (version, ifd_offset)
            }
        };

        // 验证版本号
        if version != 42 {
            return Err(format!("无效的TIFF版本号: {}, 期望42", version).into());
        }

        Ok(TiffHeader {
            byte_order,
            version,
            ifd_offset,
        })
    }
}

/// IFD标签条目
#[derive(Debug, Clone)]
pub struct IfdEntry {
    /// 标签ID
    pub tag: u16,
    /// 数据类型
    pub field_type: u16,
    /// 值的数量
    pub count: u32,
    /// 值或偏移量
    pub value_offset: u32,
}

/// IFD（Image File Directory）结构
#[derive(Debug)]
pub struct IfdData {
    /// IFD条目列表
    pub entries: Vec<IfdEntry>,
    /// 下一个IFD的偏移量（0表示无后续IFD）
    pub next_ifd_offset: u32,
}

impl IfdData {
    /// 从TIFF文件中提取IFD数据
    pub fn extract<P: AsRef<Path>>(
        filename: P,
        offset: u32,
        byte_order: ByteOrder,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        file.seek(SeekFrom::Start(offset as u64))?;

        // 读取条目数量
        let entry_count = match byte_order {
            ByteOrder::LittleEndian => file.read_u16::<LittleEndian>()?,
            ByteOrder::BigEndian => file.read_u16::<BigEndian>()?,
        };

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let entry = match byte_order {
                ByteOrder::LittleEndian => IfdEntry {
                    tag: file.read_u16::<LittleEndian>()?,
                    field_type: file.read_u16::<LittleEndian>()?,
                    count: file.read_u32::<LittleEndian>()?,
                    value_offset: file.read_u32::<LittleEndian>()?,
                },
                ByteOrder::BigEndian => IfdEntry {
                    tag: file.read_u16::<BigEndian>()?,
                    field_type: file.read_u16::<BigEndian>()?,
                    count: file.read_u32::<BigEndian>()?,
                    value_offset: file.read_u32::<BigEndian>()?,
                },
            };
            entries.push(entry);
        }

        // 读取下一个IFD偏移量
        let next_ifd_offset = match byte_order {
            ByteOrder::LittleEndian => file.read_u32::<LittleEndian>()?,
            ByteOrder::BigEndian => file.read_u32::<BigEndian>()?,
        };

        Ok(IfdData {
            entries,
            next_ifd_offset,
        })
    }
}

/// TIFF图像数据结构
#[derive(Debug)]
pub struct ImageData {
    /// 图像宽度
    pub width: u32,
    /// 图像高度
    pub height: u32,
    /// 每样本位数
    pub bits_per_sample: u16,
    /// 压缩方式 (1=无压缩, 5=LZW, 6=JPEG等)
    pub compression: u16,
    /// 图像数据strip偏移量列表
    pub strip_offsets: Vec<u32>,
    /// 每个strip的字节数
    pub strip_byte_counts: Vec<u32>,
    /// 原始像素数据
    pub data: Vec<u8>,
}

impl ImageData {
    /// 从TIFF文件中提取图像数据
    pub fn extract<P: AsRef<Path>>(
        filename: P,
        ifd: &IfdData,
        byte_order: ByteOrder,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut bits_per_sample: u16 = 8;
        let mut compression: u16 = 1;
        let mut strip_offsets: Vec<u32> = Vec::new();
        let mut strip_byte_counts: Vec<u32> = Vec::new();

        // 从IFD条目解析图像参数
        for entry in &ifd.entries {
            match entry.tag {
                // ImageWidth
                256 => width = entry.value_offset,
                // ImageLength (Height)
                257 => height = entry.value_offset,
                // BitsPerSample（当count>1时value_offset为指针，取低16位作为单通道位数）
                258 => {
                    if entry.count == 1 {
                        bits_per_sample = entry.value_offset as u16;
                    } else {
                        // 多通道图像，读取第一个样本的位数
                        let mut f = File::open(&filename)?;
                        f.seek(SeekFrom::Start(entry.value_offset as u64))?;
                        bits_per_sample = match byte_order {
                            ByteOrder::LittleEndian => f.read_u16::<LittleEndian>()?,
                            ByteOrder::BigEndian => f.read_u16::<BigEndian>()?,
                        };
                    }
                }
                // Compression
                259 => compression = entry.value_offset as u16,
                // StripOffsets
                273 => {
                    if entry.count == 1 {
                        strip_offsets.push(entry.value_offset);
                    } else {
                        strip_offsets = Self::read_offset_array(
                            &filename,
                            entry.value_offset,
                            entry.count,
                            byte_order,
                        )?;
                    }
                }
                // StripByteCounts
                279 => {
                    if entry.count == 1 {
                        strip_byte_counts.push(entry.value_offset);
                    } else {
                        strip_byte_counts = Self::read_offset_array(
                            &filename,
                            entry.value_offset,
                            entry.count,
                            byte_order,
                        )?;
                    }
                }
                _ => {}
            }
        }

        // 读取图像数据
        let mut data = Vec::new();
        let mut file = File::open(&filename)?;
        for (i, offset) in strip_offsets.iter().enumerate() {
            file.seek(SeekFrom::Start(*offset as u64))?;
            let byte_count = strip_byte_counts.get(i).copied().unwrap_or(0);
            let mut strip_data = vec![0u8; byte_count as usize];
            file.read_exact(&mut strip_data)?;
            data.extend_from_slice(&strip_data);
        }

        Ok(ImageData {
            width,
            height,
            bits_per_sample,
            compression,
            strip_offsets,
            strip_byte_counts,
            data,
        })
    }

    /// 读取偏移量数组
    fn read_offset_array<P: AsRef<Path>>(
        filename: P,
        offset: u32,
        count: u32,
        byte_order: ByteOrder,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        file.seek(SeekFrom::Start(offset as u64))?;

        let mut values = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let val = match byte_order {
                ByteOrder::LittleEndian => file.read_u32::<LittleEndian>()?,
                ByteOrder::BigEndian => file.read_u32::<BigEndian>()?,
            };
            values.push(val);
        }
        Ok(values)
    }
}

/// TIFF文件解码器主结构
#[derive(Debug)]
pub struct TiffDecoder {
    /// 文件路径
    pub filename: String,
    /// 文件头部信息
    pub header: TiffHeader,
    /// 主IFD数据
    pub ifd: IfdData,
    /// 图像数据
    pub image_data: ImageData,
}

impl TiffDecoder {
    /// 创建新的TIFF解码器实例
    pub fn new<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let filename_str = filename.as_ref().to_string_lossy().to_string();

        // 解析头部
        let header = TiffHeader::parse(&filename)?;

        // 提取主IFD
        let ifd = IfdData::extract(&filename, header.ifd_offset, header.byte_order)?;

        // 提取图像数据
        let image_data = ImageData::extract(&filename, &ifd, header.byte_order)?;

        Ok(TiffDecoder {
            filename: filename_str,
            header,
            ifd,
            image_data,
        })
    }

    /// 导出图像数据到文件
    pub fn export_data<P: AsRef<Path>>(&self, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(output_path, &self.image_data.data)?;
        Ok(())
    }

    /// 获取相机/软件信息
    pub fn get_camera_info(&self) -> String {
        // 查找Software标签 (Tag 305) 或 Make标签 (Tag 271)
        for entry in &self.ifd.entries {
            if entry.tag == 271 || entry.tag == 305 {
                return format!("Tag {}: offset={}", entry.tag, entry.value_offset);
            }
        }
        "Unknown".to_string()
    }

    /// 获取图像尺寸
    pub fn get_image_dimensions(&self) -> (u32, u32) {
        (self.image_data.width, self.image_data.height)
    }

    /// 获取图像数据大小（字节）
    pub fn get_data_size(&self) -> usize {
        self.image_data.data.len()
    }

    /// 获取IFD条目数量
    pub fn get_ifd_entry_count(&self) -> usize {
        self.ifd.entries.len()
    }

    /// 获取压缩方式描述
    pub fn get_compression_info(&self) -> &str {
        match self.image_data.compression {
            1 => "无压缩",
            2 => "CCITT Modified Huffman RLE",
            5 => "LZW",
            6 => "JPEG (旧版)",
            7 => "JPEG (新版)",
            8 => "Deflate",
            32773 => "PackBits",
            _ => "未知压缩",
        }
    }

    /// 获取每样本位数
    pub fn get_bits_per_sample(&self) -> u16 {
        self.image_data.bits_per_sample
    }

    /// 获取字节序信息
    pub fn get_byte_order(&self) -> &str {
        match self.header.byte_order {
            ByteOrder::LittleEndian => "Little Endian (Intel)",
            ByteOrder::BigEndian => "Big Endian (Motorola)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_tiff_decoder_basic() {
        let test_file = "test_data/000226830002.tif";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = TiffDecoder::new(test_file).expect("无法解码TIFF文件");

        // 验证基本信息
        let (width, height) = decoder.get_image_dimensions();
        assert!(width > 0, "图像宽度应该大于0");
        assert!(height > 0, "图像高度应该大于0");
        assert!(decoder.get_data_size() > 0, "图像数据大小应该大于0");
        assert!(decoder.get_ifd_entry_count() > 0, "IFD条目数量应该大于0");

        println!("图像尺寸: {}x{}", width, height);
        println!("数据大小: {} bytes", decoder.get_data_size());
        println!("IFD条目数: {}", decoder.get_ifd_entry_count());
        println!("压缩方式: {}", decoder.get_compression_info());
        println!("位深度: {}", decoder.get_bits_per_sample());
        println!("字节序: {}", decoder.get_byte_order());
    }

    #[test]
    fn test_tiff_header_structure() {
        let test_file = "test_data/000226830002.tif";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let header = TiffHeader::parse(test_file).expect("无法解析TIFF头部");

        // 验证TIFF版本号
        assert_eq!(header.version, 42, "TIFF版本号应该为42");

        // 验证IFD偏移量
        assert!(header.ifd_offset > 0, "IFD偏移量应该大于0");

        // 验证字节序
        assert!(
            header.byte_order == ByteOrder::LittleEndian
                || header.byte_order == ByteOrder::BigEndian,
            "字节序应该是有效值"
        );

        println!("TIFF版本: {}", header.version);
        println!("字节序: {:?}", header.byte_order);
        println!("IFD偏移: {}", header.ifd_offset);
    }

    #[test]
    fn test_tiff_data_extraction() {
        let test_file = "test_data/000226830002.tif";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = TiffDecoder::new(test_file).expect("无法解码TIFF文件");

        // 验证图像数据提取正确性
        let data_size = decoder.get_data_size();
        let (width, height) = decoder.get_image_dimensions();

        // 数据大小应该与图像尺寸相关
        assert!(data_size > 0, "图像数据不应为空");

        // 验证数据大小合理性（考虑多通道和对齐）
        if decoder.image_data.compression == 1 {
            let min_expected = (width * height) as usize;
            assert!(
                data_size >= min_expected,
                "无压缩数据大小不合理: {} < 最小期望 {}",
                data_size,
                min_expected
            );
        }

        println!("数据提取验证通过");
        println!("数据大小: {} bytes", data_size);
        println!("图像尺寸: {}x{}", width, height);
    }

    #[test]
    fn test_tiff_export_data() {
        let test_file = "test_data/000226830002.tif";
        let output_file = "test_data/tiff_output.raw";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = TiffDecoder::new(test_file).expect("无法解码TIFF文件");
        decoder.export_data(output_file).expect("无法导出图像数据");

        // 验证输出文件存在且不为空
        let output_path = PathBuf::from(output_file);
        assert!(output_path.exists(), "输出文件应该存在");
        let metadata = std::fs::metadata(&output_path).expect("无法读取输出文件元数据");
        assert!(metadata.len() > 0, "输出文件不应为空");

        // 清理测试文件
        let _ = std::fs::remove_file(output_file);
    }
}
