use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Instant, SystemTime};

use image::DynamicImage;
use lru::LruCache;

/// 缓存项，包含图片和文件修改时间
struct CacheEntry {
    image: DynamicImage,
    modified_time: SystemTime,
}

/// 带文件修改时间校验的 LRU 图片缓存
pub struct ImageCache {
    cache: Mutex<LruCache<PathBuf, CacheEntry>>,
    /// 记录缓存命中率统计
    stats: Mutex<CacheStats>,
}

#[derive(Default)]
struct CacheStats {
    hits: u64,
    misses: u64,
}

impl ImageCache {
    /// 创建指定容量的图片缓存
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(capacity.try_into().unwrap())),
            stats: Mutex::new(CacheStats::default()),
        }
    }

    /// 获取或加载图片（如果缓存未命中则使用提供的加载函数）
    pub fn get_or_load<F>(
        &self,
        path: &Path,
        loader: F,
    ) -> Result<DynamicImage, crate::error::AppError>
    where
        F: FnOnce(&Path) -> Result<DynamicImage, crate::error::AppError>,
    {
        let start = Instant::now();
        let path_str = path.to_string_lossy().to_string();

        let mut cache = self.cache.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        // 先尝试从缓存获取
        if let Some(entry) = cache.get(path) {
            // 检查文件修改时间
            if let Ok(current_modified) = Self::get_modified_time(path) {
                if entry.modified_time == current_modified {
                    // 缓存有效
                    stats.hits += 1;
                    let duration = start.elapsed();
                    println!(
                        "[ImageCache] HIT: {} | 耗时: {:?}",
                        path_str, duration
                    );
                    return Ok(entry.image.clone());
                }
                // 文件已被修改，缓存失效，继续加载
            }
        }

        // 缓存未命中或已失效，加载图片
        stats.misses += 1;
        drop(cache); // 释放锁，避免加载时持有锁
        drop(stats);

        let image = loader(path)?;

        // 存入缓存
        let modified_time = Self::get_modified_time(path)?;
        let entry = CacheEntry {
            image: image.clone(),
            modified_time,
        };

        let mut cache = self.cache.lock().unwrap();
        cache.put(path.to_path_buf(), entry);

        let duration = start.elapsed();
        println!(
            "[ImageCache] MISS: {} | 耗时: {:?}",
            path_str, duration
        );

        Ok(image)
    }

    /// 清除所有缓存
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> (u64, u64, f64) {
        let stats = self.stats.lock().unwrap();
        let total = stats.hits + stats.misses;
        let hit_rate = if total > 0 {
            (stats.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (stats.hits, stats.misses, hit_rate)
    }

    /// 获取文件修改时间
    fn get_modified_time(path: &Path) -> Result<SystemTime, crate::error::AppError> {
        let metadata = std::fs::metadata(path)?;
        metadata.modified().map_err(|e| crate::error::AppError::Io(e))
    }
}

/// 全局图片缓存实例（默认容量 50）
use once_cell::sync::Lazy;

pub static IMAGE_CACHE: Lazy<ImageCache> = Lazy::new(|| ImageCache::new(50));

/// 获取全局缓存实例
pub fn get_cache() -> &'static ImageCache {
    &IMAGE_CACHE
}
