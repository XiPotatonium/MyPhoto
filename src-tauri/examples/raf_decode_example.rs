// RAF解码器使用示例
// 运行方式: cargo run --example raf_decode_example -- <RAF文件路径> [输出JPEG路径]

use myphoto_lib::services::raw_decoders::raf_decoder::RafDecoder;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("用法: cargo run --example raf_decode_example -- <RAF文件路径> [输出JPEG路径]");
        println!("\n示例:");
        println!("  cargo run --example raf_decode_example -- sample.RAF");
        println!("  cargo run --example raf_decode_example -- sample.RAF output.jpg");
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = if args.len() > 2 {
        args[2].clone()
    } else {
        // 默认输出文件名：将 .RAF 替换为 .jpg
        let path = PathBuf::from(input_file);
        path.with_extension("jpg")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    };

    println!("\n=== RAF解码器示例 ===\n");
    println!("输入文件: {}", input_file);
    println!("输出文件: {}\n", output_file);

    // 解码RAF文件
    println!("正在解码RAF文件...");
    match RafDecoder::new(input_file) {
        Ok(decoder) => {
            println!("✓ RAF文件解码成功\n");

            // 显示文件信息
            println!("--- 文件信息 ---");
            println!("相机型号: {}", decoder.get_camera_info());
            println!("文件类型: {:?}", String::from_utf8_lossy(&decoder.header.type_string).trim_end_matches('\0'));

            // 显示JPEG信息
            println!("\n--- JPEG预览 ---");
            let jpeg_size = decoder.get_jpeg_size();
            println!("JPEG大小: {} bytes ({:.2} MB)", jpeg_size, jpeg_size as f64 / 1024.0 / 1024.0);
            println!("JPEG偏移: {} bytes", decoder.header.offset_jpg_offset);
            println!("JPEG长度: {} bytes", decoder.header.offset_jpg_length);

            // 显示CFA信息
            println!("\n--- CFA原始数据 ---");
            println!("CFA记录数: {}", decoder.get_cfa_record_count());
            println!("CFA头部偏移: {} bytes", decoder.header.offset_cfa_header_offset);
            println!("CFA头部长度: {} bytes", decoder.header.offset_cfa_header_length);
            println!("CFA数据偏移: {} bytes", decoder.header.offset_cfa_offset);
            println!("CFA数据大小: {} bytes ({:.2} MB)",
                     decoder.header.offset_cfa_length,
                     decoder.header.offset_cfa_length as f64 / 1024.0 / 1024.0);

            // 显示CFA记录详情
            println!("\n--- CFA记录详情 ---");
            for (i, record) in decoder.cfa.records.iter().enumerate() {
                println!("记录 #{}: ID=0x{:04X}, 大小={} bytes", i + 1, record.id, record.size);
            }

            // 导出JPEG
            println!("\n正在导出JPEG预览图到: {}", output_file);
            match decoder.export_jpeg(&output_file) {
                Ok(_) => {
                    println!("✓ JPEG导出成功");

                    // 验证输出文件
                    if PathBuf::from(&output_file).exists() {
                        let metadata = std::fs::metadata(&output_file).unwrap();
                        println!("  输出文件大小: {} bytes", metadata.len());
                    }
                }
                Err(e) => {
                    eprintln!("✗ JPEG导出失败: {}", e);
                    std::process::exit(1);
                }
            }

            println!("\n=== 解码完成 ===\n");
        }
        Err(e) => {
            eprintln!("✗ RAF文件解码失败: {}", e);
            eprintln!("\n可能的原因:");
            eprintln!("  1. 文件不存在");
            eprintln!("  2. 文件不是有效的RAF格式");
            eprintln!("  3. 文件已损坏");
            eprintln!("  4. 没有读取权限");
            std::process::exit(1);
        }
    }
}
