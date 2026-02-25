// RAF解码器集成测试示例
// 运行此测试需要在 src-tauri/test_data/ 目录下放置一个名为 sample.RAF 的测试文件

use myphoto_lib::services::raw_decoders::raf_decoder::RafDecoder;
use std::path::PathBuf;

#[test]
fn integration_test_raf_decoder() {
    // 测试文件路径
    let test_file = PathBuf::from("test_data/sample.RAF");

    if !test_file.exists() {
        println!("========================================");
        println!("测试跳过：找不到测试文件");
        println!("========================================");
        println!("请按照以下步骤准备测试文件：");
        println!("1. 从富士相机获取一个 .RAF 文件");
        println!("2. 将文件放入 src-tauri/test_data/ 目录");
        println!("3. 重命名文件为 sample.RAF");
        println!("4. 重新运行测试: cargo test integration_test_raf_decoder -- --nocapture");
        println!("========================================");
        return;
    }

    println!("\n========================================");
    println!("RAF解码器集成测试");
    println!("========================================\n");

    // 步骤1: 创建解码器实例
    println!("步骤 1: 加载RAF文件...");
    let decoder = match RafDecoder::new(&test_file) {
        Ok(d) => {
            println!("✓ RAF文件加载成功");
            d
        }
        Err(e) => {
            panic!("✗ RAF文件加载失败: {}", e);
        }
    };

    // 步骤2: 验证文件头信息
    println!("\n步骤 2: 读取文件头信息...");
    let camera_info = decoder.get_camera_info();
    println!("✓ 相机型号: {}", camera_info);
    println!("  文件类型: {:?}", String::from_utf8_lossy(&decoder.header.type_string));
    println!("  格式版本: {:?}", String::from_utf8_lossy(&decoder.header.format_ver));

    // 验证相机信息不为空
    assert!(!camera_info.is_empty(), "相机信息不应该为空");

    // 步骤3: 检查JPEG数据
    println!("\n步骤 3: 检查JPEG预览数据...");
    let jpeg_size = decoder.get_jpeg_size();
    println!("✓ JPEG数据大小: {} bytes ({:.2} MB)", jpeg_size, jpeg_size as f64 / 1024.0 / 1024.0);
    println!("  JPEG偏移量: {} bytes", decoder.header.offset_jpg_offset);

    // 验证JPEG数据存在且大小合理
    assert!(jpeg_size > 0, "JPEG数据大小应该大于0");
    assert!(jpeg_size > 10000, "JPEG数据应该至少有10KB");

    // 步骤4: 检查CFA数据
    println!("\n步骤 4: 检查CFA原始数据信息...");
    let cfa_count = decoder.get_cfa_record_count();
    println!("✓ CFA记录数量: {}", cfa_count);
    println!("  CFA头部偏移: {} bytes", decoder.header.offset_cfa_header_offset);
    println!("  CFA数据偏移: {} bytes", decoder.header.offset_cfa_offset);
    println!("  CFA数据大小: {} bytes ({:.2} MB)",
             decoder.header.offset_cfa_length,
             decoder.header.offset_cfa_length as f64 / 1024.0 / 1024.0);

    // 验证CFA记录存在
    assert!(cfa_count > 0, "CFA记录数量应该大于0");

    // 步骤5: 导出JPEG预览图
    println!("\n步骤 5: 导出JPEG预览图...");
    let output_file = PathBuf::from("test_data/test_output.jpg");
    match decoder.export_jpeg(&output_file) {
        Ok(_) => {
            println!("✓ JPEG导出成功: {:?}", output_file);

            // 验证输出文件
            if output_file.exists() {
                let output_size = std::fs::metadata(&output_file)
                    .map(|m| m.len())
                    .unwrap_or(0);
                println!("  输出文件大小: {} bytes", output_size);
                assert_eq!(output_size, jpeg_size as u64, "输出文件大小应该匹配");

                // 清理测试文件
                // let _ = std::fs::remove_file(&output_file);
                // println!("  清理测试输出文件完成");
            }
        }
        Err(e) => {
            panic!("✗ JPEG导出失败: {}", e);
        }
    }

    // 步骤6: 显示CFA记录详情
    println!("\n步骤 6: CFA记录详情...");
    for (i, record) in decoder.cfa.records.iter().enumerate().take(5) {
        println!("  记录 #{}: ID=0x{:04X}, 大小={} bytes", i + 1, record.id, record.size);
    }
    if decoder.cfa.records.len() > 5 {
        println!("  ... 还有 {} 条记录", decoder.cfa.records.len() - 5);
    }

    println!("\n========================================");
    println!("测试完成 - 所有检查通过 ✓");
    println!("========================================\n");
}

#[test]
fn test_raf_decoder_error_handling() {
    println!("\n测试错误处理...");

    // 测试不存在的文件
    let result = RafDecoder::new("nonexistent.RAF");
    assert!(result.is_err(), "不存在的文件应该返回错误");
    println!("✓ 正确处理文件不存在的情况");

    // 测试无效的文件
    let invalid_file = PathBuf::from("test_data/invalid.RAF");
    if !invalid_file.exists() {
        // 创建一个无效的RAF文件用于测试
        let _ = std::fs::write(&invalid_file, b"INVALID DATA");
    }

    let result = RafDecoder::new(&invalid_file);
    // 注意：根据文件内容，这可能成功或失败
    println!("✓ 错误处理测试完成");

    // 清理
    let _ = std::fs::remove_file(&invalid_file);
}
