use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{BigEndian, ReadBytesExt};

/// RAF文件头结构
#[derive(Debug)]
pub struct RafHeader {
    pub type_string: [u8; 16],
    pub format_ver: [u8; 4],
    pub camera_id: [u8; 8],
    pub camera_str: String,
    pub offset_ver: [u8; 4],
    pub offset_unk: String,
    pub offset_jpg_offset: i32,
    pub offset_jpg_length: i32,
    pub offset_cfa_header_offset: i32,
    pub offset_cfa_header_length: i32,
    pub offset_cfa_offset: i32,
    pub offset_cfa_length: i32,
}

impl RafHeader {
    /// 从RAF文件中解析头部信息
    pub fn parse<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        
        // 读取type_string (16 bytes)
        let mut type_string = [0u8; 16];
        file.read_exact(&mut type_string)?;
        
        // 读取format_ver (4 bytes)
        let mut format_ver = [0u8; 4];
        file.read_exact(&mut format_ver)?;
        
        // 读取camera_id (8 bytes)
        let mut camera_id = [0u8; 8];
        file.read_exact(&mut camera_id)?;
        
        // 读取camera_str (32 bytes)
        let mut camera_str_bytes = [0u8; 32];
        file.read_exact(&mut camera_str_bytes)?;
        let camera_str = String::from_utf8_lossy(&camera_str_bytes)
            .trim_end_matches('\0')
            .to_string();
        
        // 读取offset_ver (4 bytes)
        let mut offset_ver = [0u8; 4];
        file.read_exact(&mut offset_ver)?;
        
        // 读取offset_unk (20 bytes)
        let mut offset_unk_bytes = [0u8; 20];
        file.read_exact(&mut offset_unk_bytes)?;
        let offset_unk = String::from_utf8_lossy(&offset_unk_bytes)
            .trim_end_matches('\0')
            .to_string();
        
        // 读取各种偏移量和长度 (big endian)
        let offset_jpg_offset = file.read_i32::<BigEndian>()?;
        let offset_jpg_length = file.read_i32::<BigEndian>()?;
        let offset_cfa_header_offset = file.read_i32::<BigEndian>()?;
        let offset_cfa_header_length = file.read_i32::<BigEndian>()?;
        let offset_cfa_offset = file.read_i32::<BigEndian>()?;
        let offset_cfa_length = file.read_i32::<BigEndian>()?;
        
        Ok(RafHeader {
            type_string,
            format_ver,
            camera_id,
            camera_str,
            offset_ver,
            offset_unk,
            offset_jpg_offset,
            offset_jpg_length,
            offset_cfa_header_offset,
            offset_cfa_header_length,
            offset_cfa_offset,
            offset_cfa_length,
        })
    }
}

/// JPEG数据结构（包含EXIF和预览）
#[derive(Debug)]
pub struct JpegData {
    pub data: Vec<u8>,
}

impl JpegData {
    /// 从RAF文件中提取JPEG数据
    pub fn extract<P: AsRef<Path>>(
        filename: P,
        offset: i32,
        length: i32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        file.seek(SeekFrom::Start(offset as u64))?;
        
        let mut data = vec![0u8; length as usize];
        file.read_exact(&mut data)?;
        
        Ok(JpegData { data })
    }
    
    /// 将JPEG数据保存到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path, &self.data)?;
        Ok(())
    }
}

/// CFA记录结构
#[derive(Debug)]
pub struct CfaRecord {
    pub id: u16,
    pub size: u16,
    pub data: Vec<u8>,
}

/// CFA数据结构（Color Filter Array - 原始传感器数据）
#[derive(Debug)]
pub struct CfaData {
    pub count: i32,
    pub records: Vec<CfaRecord>,
    pub data_offset: i32,
    pub data_length: i32,
}

impl CfaData {
    /// 从RAF文件中提取CFA数据
    pub fn extract<P: AsRef<Path>>(
        filename: P,
        header_offset: i32,
        _header_length: i32,
        data_offset: i32,
        data_length: i32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        file.seek(SeekFrom::Start(header_offset as u64))?;
        
        // 读取记录数量
        let count = file.read_i32::<BigEndian>()?;
        
        let mut records = Vec::new();
        for _ in 0..count {
            let id = file.read_u16::<BigEndian>()?;
            let size = file.read_u16::<BigEndian>()?;
            
            let mut data = vec![0u8; size as usize];
            file.read_exact(&mut data)?;
            
            records.push(CfaRecord { id, size, data });
        }
        
        Ok(CfaData {
            count,
            records,
            data_offset,
            data_length,
        })
    }
}

/// RAF文件解码器主结构
#[derive(Debug)]
pub struct RafDecoder {
    pub filename: String,
    pub header: RafHeader,
    pub jpeg: JpegData,
    pub cfa: CfaData,
}

impl RafDecoder {
    /// 创建新的RAF解码器实例
    pub fn new<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
        let filename_str = filename.as_ref().to_string_lossy().to_string();
        
        // 解析头部
        let header = RafHeader::parse(&filename)?;
        
        // 提取JPEG数据
        let jpeg = JpegData::extract(
            &filename,
            header.offset_jpg_offset,
            header.offset_jpg_length,
        )?;
        
        // 提取CFA数据
        let cfa = CfaData::extract(
            &filename,
            header.offset_cfa_header_offset,
            header.offset_cfa_header_length,
            header.offset_cfa_offset,
            header.offset_cfa_length,
        )?;
        
        Ok(RafDecoder {
            filename: filename_str,
            header,
            jpeg,
            cfa,
        })
    }
    
    /// 导出JPEG文件（包含EXIF和预览）
    pub fn export_jpeg<P: AsRef<Path>>(&self, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.jpeg.save(output_path)?;
        Ok(())
    }
    
    /// 获取相机信息
    pub fn get_camera_info(&self) -> &str {
        &self.header.camera_str
    }
    
    /// 获取JPEG数据大小
    pub fn get_jpeg_size(&self) -> usize {
        self.jpeg.data.len()
    }
    
    /// 获取CFA记录数量
    pub fn get_cfa_record_count(&self) -> i32 {
        self.cfa.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_raf_decoder_basic() {
        // 这个测试需要一个真实的RAF文件
        // 您需要提供一个测试RAF文件的路径
        let test_file = "test_data/sample.RAF";
        
        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }
        
        let decoder = RafDecoder::new(test_file).expect("无法解码RAF文件");
        
        // 验证基本信息
        assert!(!decoder.get_camera_info().is_empty(), "相机信息应该不为空");
        assert!(decoder.get_jpeg_size() > 0, "JPEG数据大小应该大于0");
        assert!(decoder.get_cfa_record_count() > 0, "CFA记录数量应该大于0");
        
        println!("相机: {}", decoder.get_camera_info());
        println!("JPEG大小: {} bytes", decoder.get_jpeg_size());
        println!("CFA记录数: {}", decoder.get_cfa_record_count());
    }
    
    #[test]
    fn test_export_jpeg() {
        let test_file = "test_data/sample.RAF";
        let output_file = "test_data/output.jpg";
        
        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }
        
        let decoder = RafDecoder::new(test_file).expect("无法解码RAF文件");
        decoder.export_jpeg(output_file).expect("无法导出JPEG文件");
        
        // 验证输出文件存在
        assert!(PathBuf::from(output_file).exists(), "输出文件应该存在");
        
        // 清理测试文件
        let _ = std::fs::remove_file(output_file);
    }
    
    #[test]
    fn test_raf_header_structure() {
        // 测试RAF头部结构的基本功能
        // 这是一个结构测试，不需要实际文件
        let test_file = "test_data/sample.RAF";
        
        if !PathBuf::from(test_file).exists() {
            println!("跳过测试：测试文件不存在 {}", test_file);
            return;
        }
        
        let header = RafHeader::parse(test_file).expect("无法解析RAF头部");
        
        // 验证RAF魔数（前16字节应该包含"FUJIFILMCCD-RAW"）
        let type_str = String::from_utf8_lossy(&header.type_string);
        assert!(type_str.contains("FUJIFILM"), "类型字符串应该包含FUJIFILM");
        
        // 验证偏移量是正数
        assert!(header.offset_jpg_offset > 0, "JPEG偏移量应该是正数");
        assert!(header.offset_jpg_length > 0, "JPEG长度应该是正数");
        
        println!("RAF类型: {:?}", type_str);
        println!("相机: {}", header.camera_str);
        println!("JPEG偏移: {}, 长度: {}", header.offset_jpg_offset, header.offset_jpg_length);
    }
}
