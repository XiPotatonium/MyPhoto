use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use image::DynamicImage;
use lru::LruCache;
use tokio::sync::{Mutex, broadcast};

/// 缓存项，包含图片和文件修改时间
struct CacheEntry {
    image: Arc<DynamicImage>,
    modified_time: SystemTime,
}

/// 带文件修改时间校验的 LRU 图片缓存，支持 in-flight 加载去重
pub struct ImageCache {
    cache: Mutex<LruCache<PathBuf, CacheEntry>>,
    /// 正在加载中的图片：路径 → broadcast::Sender，后续请求可订阅同一 channel 等待结果
    in_flight: Mutex<HashMap<PathBuf, broadcast::Sender<Arc<DynamicImage>>>>,
    /// 记录缓存命中率统计
    stats: std::sync::Mutex<CacheStats>,
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
            in_flight: Mutex::new(HashMap::new()),
            stats: std::sync::Mutex::new(CacheStats::default()),
        }
    }

    /// 异步获取或加载图片（支持 in-flight 去重，避免同一图片被并发重复加载）
    pub async fn get_or_load<F>(
        &self,
        path: &Path,
        loader: F,
    ) -> Result<Arc<DynamicImage>, crate::error::AppError>
    where
        F: FnOnce(&Path) -> Result<DynamicImage, crate::error::AppError> + Send + 'static,
    {
        let start = Instant::now();
        let path_str = path.to_string_lossy().to_string();

        // 1. 检查已完成的缓存
        {
            let mut cache = self.cache.lock().await;
            if let Some(entry) = cache.get(path) {
                if let Ok(current_modified) = Self::get_modified_time(path) {
                    if entry.modified_time == current_modified {
                        let mut stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
                        stats.hits += 1;
                        log::debug!(
                            "[ImageCache] HIT: {} | 耗时: {:?}",
                            path_str,
                            start.elapsed()
                        );
                        return Ok(entry.image.clone());
                    }
                }
            }
        }

        // 2. 检查是否有正在加载中的同一图片（in-flight 去重）
        {
            let in_flight = self.in_flight.lock().await;
            if let Some(sender) = in_flight.get(path) {
                let mut rx = sender.subscribe();
                drop(in_flight);
                log::debug!(
                    "[ImageCache] DEDUP: waiting for in-flight load of {}",
                    path_str
                );
                return rx.recv().await.map_err(|_| {
                    crate::error::AppError::General("In-flight image load failed".into())
                });
            }
        }

        // 3. 注册新的加载任务
        let (tx, _initial_rx) = broadcast::channel::<Arc<DynamicImage>>(1);
        let path_buf = path.to_path_buf();
        {
            let mut in_flight = self.in_flight.lock().await;
            in_flight.insert(path_buf.clone(), tx.clone());
        }

        // 4. 在阻塞线程池中执行 I/O 密集的图片加载
        let path_for_load = path_buf.clone();
        let load_result = tokio::task::spawn_blocking(move || loader(&path_for_load))
            .await
            .map_err(|e| crate::error::AppError::General(format!("Task join error: {}", e)))?;

        // 先广播结果给等待者，再从 in_flight 移除（确保等待者能收到）
        match load_result {
            Ok(image) => {
                let arc_image = Arc::new(image);
                let _ = tx.send(arc_image.clone());

                {
                    let mut in_flight = self.in_flight.lock().await;
                    in_flight.remove(&path_buf);
                }

                // 存入缓存
                let modified_time = Self::get_modified_time(path)?;
                let mut cache = self.cache.lock().await;
                cache.put(
                    path_buf,
                    CacheEntry {
                        image: arc_image.clone(),
                        modified_time,
                    },
                );

                let mut stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
                stats.misses += 1;
                log::debug!(
                    "[ImageCache] MISS: {} | 耗时: {:?}",
                    path_str,
                    start.elapsed()
                );
                Ok(arc_image)
            }
            Err(e) => {
                {
                    let mut in_flight = self.in_flight.lock().await;
                    in_flight.remove(&path_buf);
                }
                // tx 被 drop，等待者会收到 RecvError
                Err(e)
            }
        }
    }

    /// 清除所有缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> (u64, u64, f64) {
        let stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
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
pub static IMAGE_CACHE: std::sync::LazyLock<ImageCache> = std::sync::LazyLock::new(|| ImageCache::new(50));

/// 获取全局缓存实例
pub fn get_cache() -> &'static ImageCache {
    &IMAGE_CACHE
}
