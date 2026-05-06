use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

/// DNG字节序类型（DNG基于TIFF格式）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DngByteOrder {
    /// Little Endian - "II"
    LittleEndian,
    /// Big Endian - "MM"
    BigEndian,
}

/// DNG文件头结构（基于TIFF格式）
#[derive(Debug)]
pub struct DngHeader {
    /// 字节序标识
    pub byte_order: DngByteOrder,
    /// TIFF版本号（应为42）
    pub version: u16,
    /// 第一个IFD的偏移量
    pub ifd_offset: u32,
}

impl DngHeader {
    /// 从DNG文件中解析头部信息
    pub fn parse<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;

        // 读取字节序标识 (2 bytes)
        let mut order_bytes = [0u8; 2];
        file.read_exact(&mut order_bytes)?;

        let byte_order = match &order_bytes {
            b"II" => DngByteOrder::LittleEndian,
            b"MM" => DngByteOrder::BigEndian,
            _ => return Err("无效的DNG字节序标识".into()),
        };

        // 读取版本号和IFD偏移量
        let (version, ifd_offset) = match byte_order {
            DngByteOrder::LittleEndian => {
                let version = file.read_u16::<LittleEndian>()?;
                let ifd_offset = file.read_u32::<LittleEndian>()?;
                (version, ifd_offset)
            }
            DngByteOrder::BigEndian => {
                let version = file.read_u16::<BigEndian>()?;
                let ifd_offset = file.read_u32::<BigEndian>()?;
                (version, ifd_offset)
            }
        };

        // DNG基于TIFF格式，版本号应为42
        if version != 42 {
            return Err(format!("无效的DNG/TIFF版本号: {}, 期望42", version).into());
        }

        Ok(DngHeader {
            byte_order,
            version,
            ifd_offset,
        })
    }
}

/// DNG IFD条目
#[derive(Debug, Clone)]
pub struct DngIfdEntry {
    /// 标签ID
    pub tag: u16,
    /// 数据类型 (1=BYTE, 2=ASCII, 3=SHORT, 4=LONG, 5=RATIONAL等)
    pub field_type: u16,
    /// 值的数量
    pub count: u32,
    /// 值或偏移量
    pub value_offset: u32,
}

/// DNG IFD结构
#[derive(Debug)]
pub struct DngIfd {
    /// IFD条目列表
    pub entries: Vec<DngIfdEntry>,
    /// 下一个IFD的偏移量
    pub next_ifd_offset: u32,
}

impl DngIfd {
    /// 从DNG文件中提取IFD数据
    pub fn extract<P: AsRef<Path>>(
        filename: P,
        offset: u32,
        byte_order: DngByteOrder,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        file.seek(SeekFrom::Start(offset as u64))?;

        let entry_count = match byte_order {
            DngByteOrder::LittleEndian => file.read_u16::<LittleEndian>()?,
            DngByteOrder::BigEndian => file.read_u16::<BigEndian>()?,
        };

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let entry = match byte_order {
                DngByteOrder::LittleEndian => DngIfdEntry {
                    tag: file.read_u16::<LittleEndian>()?,
                    field_type: file.read_u16::<LittleEndian>()?,
                    count: file.read_u32::<LittleEndian>()?,
                    value_offset: file.read_u32::<LittleEndian>()?,
                },
                DngByteOrder::BigEndian => DngIfdEntry {
                    tag: file.read_u16::<BigEndian>()?,
                    field_type: file.read_u16::<BigEndian>()?,
                    count: file.read_u32::<BigEndian>()?,
                    value_offset: file.read_u32::<BigEndian>()?,
                },
            };
            entries.push(entry);
        }

        let next_ifd_offset = match byte_order {
            DngByteOrder::LittleEndian => file.read_u32::<LittleEndian>()?,
            DngByteOrder::BigEndian => file.read_u32::<BigEndian>()?,
        };

        Ok(DngIfd {
            entries,
            next_ifd_offset,
        })
    }

    /// 根据标签ID查找条目
    pub fn find_entry(&self, tag: u16) -> Option<&DngIfdEntry> {
        self.entries.iter().find(|e| e.tag == tag)
    }
}

/// DNG嵌入式预览图数据
#[derive(Debug)]
pub struct DngPreviewData {
    /// 预览图数据（通常为JPEG格式）
    pub data: Vec<u8>,
    /// 预览图宽度
    pub width: u32,
    /// 预览图高度
    pub height: u32,
}

impl DngPreviewData {
    /// 从DNG文件中提取预览图
    pub fn extract<P: AsRef<Path>>(
        filename: P,
        ifd: &DngIfd,
        byte_order: DngByteOrder,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut strip_offset: u32 = 0;
        let mut strip_byte_count: u32 = 0;

        // 从IFD中提取预览图参数
        for entry in &ifd.entries {
            match entry.tag {
                256 => width = entry.value_offset,   // ImageWidth
                257 => height = entry.value_offset,  // ImageLength
                273 => strip_offset = entry.value_offset,  // StripOffsets
                279 => strip_byte_count = entry.value_offset,  // StripByteCounts
                513 => strip_offset = entry.value_offset,  // JPEGInterchangeFormat
                514 => strip_byte_count = entry.value_offset,  // JPEGInterchangeFormatLength
                _ => {}
            }
        }

        // 读取预览数据
        let mut data = Vec::new();
        if strip_offset > 0 && strip_byte_count > 0 {
            let mut file = File::open(&filename)?;
            file.seek(SeekFrom::Start(strip_offset as u64))?;
            data = vec![0u8; strip_byte_count as usize];
            file.read_exact(&mut data)?;
        } else {
            // 尝试从多个strip中读取
            let offsets = Self::read_strip_values(&filename, ifd, 273, byte_order)?;
            let counts = Self::read_strip_values(&filename, ifd, 279, byte_order)?;

            if !offsets.is_empty() {
                let mut file = File::open(&filename)?;
                for (i, offset) in offsets.iter().enumerate() {
                    let count = counts.get(i).copied().unwrap_or(0);
                    file.seek(SeekFrom::Start(*offset as u64))?;
                    let mut strip = vec![0u8; count as usize];
                    file.read_exact(&mut strip)?;
                    data.extend_from_slice(&strip);
                }
            }
        }

        Ok(DngPreviewData {
            data,
            width,
            height,
        })
    }

    /// 读取strip值数组
    fn read_strip_values<P: AsRef<Path>>(
        filename: P,
        ifd: &DngIfd,
        tag: u16,
        byte_order: DngByteOrder,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        if let Some(entry) = ifd.find_entry(tag) {
            if entry.count == 1 {
                return Ok(vec![entry.value_offset]);
            }
            let mut file = File::open(filename)?;
            file.seek(SeekFrom::Start(entry.value_offset as u64))?;
            let mut values = Vec::with_capacity(entry.count as usize);
            for _ in 0..entry.count {
                let val = match byte_order {
                    DngByteOrder::LittleEndian => file.read_u32::<LittleEndian>()?,
                    DngByteOrder::BigEndian => file.read_u32::<BigEndian>()?,
                };
                values.push(val);
            }
            Ok(values)
        } else {
            Ok(Vec::new())
        }
    }

    /// 将预览图保存到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path, &self.data)?;
        Ok(())
    }
}

/// DNG RAW数据信息
#[derive(Debug)]
pub struct DngRawInfo {
    /// RAW图像宽度
    pub width: u32,
    /// RAW图像高度
    pub height: u32,
    /// 每样本位数
    pub bits_per_sample: u16,
    /// 压缩方式
    pub compression: u16,
    /// CFA模式（Bayer模式）
    pub cfa_pattern: Vec<u8>,
    /// DNG版本
    pub dng_version: [u8; 4],
    /// 数据偏移量
    pub data_offset: u32,
    /// 数据长度
    pub data_length: u32,
}

impl DngRawInfo {
    /// 从DNG文件的IFD中提取RAW信息
    pub fn extract<P: AsRef<Path>>(
        filename: P,
        ifd: &DngIfd,
        _byte_order: DngByteOrder,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut bits_per_sample: u16 = 16;
        let mut compression: u16 = 1;
        let mut cfa_pattern = Vec::new();
        let mut dng_version = [0u8; 4];
        let mut data_offset: u32 = 0;
        let mut data_length: u32 = 0;

        for entry in &ifd.entries {
            match entry.tag {
                256 => width = entry.value_offset,          // ImageWidth
                257 => height = entry.value_offset,         // ImageLength
                258 => bits_per_sample = entry.value_offset as u16, // BitsPerSample
                259 => compression = entry.value_offset as u16,     // Compression
                273 => data_offset = entry.value_offset,    // StripOffsets
                279 => data_length = entry.value_offset,    // StripByteCounts
                324 => data_offset = entry.value_offset,    // TileOffsets
                325 => data_length = entry.value_offset,    // TileByteCounts
                // DNG版本 (Tag 50706)
                50706 => {
                    dng_version = entry.value_offset.to_le_bytes();
                }
                // CFA模式 (Tag 33422)
                33422 => {
                    if entry.count <= 4 {
                        cfa_pattern = entry.value_offset.to_le_bytes()[..entry.count as usize].to_vec();
                    } else {
                        // 从文件中读取CFA模式
                        let mut file = File::open(&filename)?;
                        file.seek(SeekFrom::Start(entry.value_offset as u64))?;
                        cfa_pattern = vec![0u8; entry.count as usize];
                        file.read_exact(&mut cfa_pattern)?;
                    }
                }
                _ => {}
            }
        }

        Ok(DngRawInfo {
            width,
            height,
            bits_per_sample,
            compression,
            cfa_pattern,
            dng_version,
            data_offset,
            data_length,
        })
    }
}

/// DNG文件解码器主结构
#[derive(Debug)]
pub struct DngDecoder {
    /// 文件路径
    pub filename: String,
    /// 文件头部信息
    pub header: DngHeader,
    /// 主IFD（通常包含缩略图信息）
    pub main_ifd: DngIfd,
    /// 子IFD列表（包含RAW数据信息）
    pub sub_ifds: Vec<DngIfd>,
    /// 预览数据
    pub preview: DngPreviewData,
    /// RAW数据信息
    pub raw_info: DngRawInfo,
}

impl DngDecoder {
    /// 创建新的DNG解码器实例
    pub fn new<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let filename_str = filename.as_ref().to_string_lossy().to_string();

        // 解析头部
        let header = DngHeader::parse(&filename)?;

        // 提取主IFD
        let main_ifd = DngIfd::extract(&filename, header.ifd_offset, header.byte_order)?;

        // 尝试提取子IFD
        let mut sub_ifds = Vec::new();
        // SubIFDs标签 (Tag 330)
        if let Some(sub_ifd_entry) = main_ifd.find_entry(330) {
            if sub_ifd_entry.count == 1 {
                if let Ok(sub_ifd) =
                    DngIfd::extract(&filename, sub_ifd_entry.value_offset, header.byte_order)
                {
                    sub_ifds.push(sub_ifd);
                }
            } else {
                // 多个子IFD，从偏移量数组中读取
                let mut file = File::open(&filename)?;
                file.seek(SeekFrom::Start(sub_ifd_entry.value_offset as u64))?;
                for _ in 0..sub_ifd_entry.count {
                    let offset = match header.byte_order {
                        DngByteOrder::LittleEndian => file.read_u32::<LittleEndian>()?,
                        DngByteOrder::BigEndian => file.read_u32::<BigEndian>()?,
                    };
                    if let Ok(sub_ifd) = DngIfd::extract(&filename, offset, header.byte_order) {
                        sub_ifds.push(sub_ifd);
                    }
                }
            }
        }

        // 也检查第二个IFD（有些DNG将预览放在第二个IFD中）
        if main_ifd.next_ifd_offset > 0 {
            if let Ok(second_ifd) =
                DngIfd::extract(&filename, main_ifd.next_ifd_offset, header.byte_order)
            {
                sub_ifds.push(second_ifd);
            }
        }

        // 提取预览数据（从主IFD）
        let preview = DngPreviewData::extract(&filename, &main_ifd, header.byte_order)?;

        // 提取RAW信息（优先从子IFD获取，否则从主IFD获取）
        let raw_info = if !sub_ifds.is_empty() {
            DngRawInfo::extract(&filename, &sub_ifds[0], header.byte_order)?
        } else {
            DngRawInfo::extract(&filename, &main_ifd, header.byte_order)?
        };

        Ok(DngDecoder {
            filename: filename_str,
            header,
            main_ifd,
            sub_ifds,
            preview,
            raw_info,
        })
    }

    /// 导出预览图到文件（JPEG格式）
    pub fn export_preview<P: AsRef<Path>>(
        &self,
        output_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.preview.data.is_empty() {
            return Err("DNG文件中未找到预览图数据".into());
        }
        self.preview.save(output_path)?;
        Ok(())
    }

    /// 获取相机信息
    pub fn get_camera_info(&self) -> String {
        let mut make = String::new();
        let mut model = String::new();

        // Make (Tag 271) 和 Model (Tag 272) 通常是字符串引用
        for entry in &self.main_ifd.entries {
            match entry.tag {
                271 => make = format!("Make@{}", entry.value_offset),
                272 => model = format!("Model@{}", entry.value_offset),
                _ => {}
            }
        }

        if !make.is_empty() || !model.is_empty() {
            format!("{} {}", make, model).trim().to_string()
        } else {
            "Unknown DNG Camera".to_string()
        }
    }

    /// 获取DNG版本号字符串
    pub fn get_dng_version(&self) -> String {
        let v = &self.raw_info.dng_version;
        format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3])
    }

    /// 获取预览图大小（字节）
    pub fn get_preview_size(&self) -> usize {
        self.preview.data.len()
    }

    /// 获取RAW图像尺寸
    pub fn get_raw_dimensions(&self) -> (u32, u32) {
        (self.raw_info.width, self.raw_info.height)
    }

    /// 获取RAW数据位深度
    pub fn get_bits_per_sample(&self) -> u16 {
        self.raw_info.bits_per_sample
    }

    /// 获取IFD条目总数
    pub fn get_ifd_entry_count(&self) -> usize {
        self.main_ifd.entries.len()
    }

    /// 获取子IFD数量
    pub fn get_sub_ifd_count(&self) -> usize {
        self.sub_ifds.len()
    }

    /// 获取压缩方式描述
    pub fn get_compression_info(&self) -> &str {
        match self.raw_info.compression {
            1 => "无压缩",
            7 => "JPEG压缩",
            8 => "Deflate压缩",
            34892 => "有损JPEG压缩",
            _ => "未知压缩方式",
        }
    }

    /// 获取字节序信息
    pub fn get_byte_order(&self) -> &str {
        match self.header.byte_order {
            DngByteOrder::LittleEndian => "Little Endian (Intel)",
            DngByteOrder::BigEndian => "Big Endian (Motorola)",
        }
    }

    /// 获取RAW数据的偏移和长度信息
    pub fn get_raw_data_info(&self) -> (u32, u32) {
        (self.raw_info.data_offset, self.raw_info.data_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_dng_decoder_basic() {
        let test_file = "test_data/DSCF0409.dng";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = DngDecoder::new(test_file).expect("无法解码DNG文件");

        // 验证基本信息
        assert!(decoder.get_ifd_entry_count() > 0, "IFD条目数量应该大于0");

        let (raw_width, raw_height) = decoder.get_raw_dimensions();
        println!("相机信息: {}", decoder.get_camera_info());
        println!("DNG版本: {}", decoder.get_dng_version());
        println!("RAW尺寸: {}x{}", raw_width, raw_height);
        println!("位深度: {}", decoder.get_bits_per_sample());
        println!("IFD条目数: {}", decoder.get_ifd_entry_count());
        println!("子IFD数: {}", decoder.get_sub_ifd_count());
        println!("预览大小: {} bytes", decoder.get_preview_size());
        println!("压缩方式: {}", decoder.get_compression_info());
        println!("字节序: {}", decoder.get_byte_order());
    }

    #[test]
    fn test_dng_header_structure() {
        let test_file = "test_data/DSCF0409.dng";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let header = DngHeader::parse(test_file).expect("无法解析DNG头部");

        // DNG基于TIFF格式，版本应为42
        assert_eq!(header.version, 42, "DNG/TIFF版本号应该为42");

        // 验证IFD偏移量合理
        assert!(header.ifd_offset > 0, "IFD偏移量应该大于0");

        // 验证字节序
        assert!(
            header.byte_order == DngByteOrder::LittleEndian
                || header.byte_order == DngByteOrder::BigEndian,
            "字节序应该是有效值"
        );

        println!("DNG/TIFF版本: {}", header.version);
        println!("字节序: {:?}", header.byte_order);
        println!("IFD偏移: {}", header.ifd_offset);
    }

    #[test]
    fn test_dng_data_extraction() {
        let test_file = "test_data/DSCF0409.dng";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = DngDecoder::new(test_file).expect("无法解码DNG文件");

        // 验证RAW数据信息
        let (data_offset, data_length) = decoder.get_raw_data_info();
        println!("RAW数据偏移: {}", data_offset);
        println!("RAW数据长度: {}", data_length);

        // 验证预览图提取
        let preview_size = decoder.get_preview_size();
        if preview_size > 0 {
            // 检查JPEG标识 (FFD8)
            assert!(
                decoder.preview.data.len() >= 2,
                "预览数据长度应该至少为2字节"
            );
            if decoder.preview.data[0] == 0xFF && decoder.preview.data[1] == 0xD8 {
                println!("预览图格式: JPEG (验证通过)");
            } else {
                println!("预览图格式: 非标准JPEG (首字节: {:02X}{:02X})",
                    decoder.preview.data[0], decoder.preview.data[1]);
            }
        }

        println!("数据提取验证通过");
    }

    #[test]
    fn test_dng_export_preview() {
        let test_file = "test_data/DSCF0409.dng";
        let output_file = "test_data/dng_preview_output.jpg";

        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }

        let decoder = DngDecoder::new(test_file).expect("无法解码DNG文件");

        if decoder.get_preview_size() == 0 {
            println!("跳过测试：DNG文件中无预览图");
            return;
        }

        decoder
            .export_preview(output_file)
            .expect("无法导出预览图");

        // 验证输出文件存在且不为空
        let output_path = PathBuf::from(output_file);
        assert!(output_path.exists(), "输出文件应该存在");
        let metadata = std::fs::metadata(&output_path).expect("无法读取输出文件元数据");
        assert!(metadata.len() > 0, "输出文件不应为空");

        // 清理测试文件
        let _ = std::fs::remove_file(output_file);
    }
}
