// RAW图像格式解码器模块
//
// 此模块包含各种相机RAW格式的解码器实现
// 每个子模块对应一种RAW格式

/// 富士RAF格式解码器
pub mod raf_decoder;

// 未来可以添加更多RAW格式解码器：
// pub mod cr2_decoder;  // Canon RAW
// pub mod nef_decoder;  // Nikon RAW
// pub mod arw_decoder;  // Sony RAW
// pub mod dng_decoder;  // Adobe DNG
// pub mod orf_decoder;  // Olympus RAW
// pub mod rw2_decoder;  // Panasonic RAW
