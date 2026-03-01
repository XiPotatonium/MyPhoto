<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ImageGroup } from '../../types/image'
import type { ExifInfo } from '../../types/exif'
import { formatGps } from '../../utils/formatters'

const props = defineProps<{
  image: ImageGroup | null
}>()

const emit = defineEmits<{
  'rating-updated': [image: ImageGroup, rating: number]
}>()

const exif = ref<ExifInfo | null>(null)
const loading = ref(false)

async function loadExif() {
  if (!props.image) {
    exif.value = null
    return
  }
  const filePath = props.image.jpgPath || props.image.rawPath
  if (!filePath) {
    exif.value = null
    return
  }
  loading.value = true
  try {
    exif.value = await invoke<ExifInfo>('read_exif', { filePath })
  } catch (e) {
    console.error('Failed to read EXIF:', e)
    exif.value = null
  } finally {
    loading.value = false
  }
}

async function setRating(value: number) {
  if (!props.image || !exif.value) return
  const filePath = props.image.jpgPath || props.image.rawPath
  if (!filePath) return
  loading.value = true
  try {
    await invoke('write_rating', { filePath, rating: value })
    exif.value.rating = value
    // 通知父组件评分已更新，同步更新前端图片信息
    emit('rating-updated', props.image, value)
  } catch (e) {
    console.error('Failed to write rating:', e)
  } finally {
    loading.value = false
  }
}

watch(() => props.image, loadExif, { immediate: true })

defineExpose({ setRating })
</script>

<template>
  <div class="exif-display">
    <h3 class="section-title">图像信息</h3>
    <div v-if="!image" class="exif-empty">未选择图像</div>
    <div v-else-if="loading" class="exif-loading">读取中...</div>
    <div v-else-if="exif" class="exif-list">
      <div class="exif-item" v-if="image.baseName">
        <span class="exif-label">文件名</span>
        <span class="exif-value">{{ image.baseName }}</span>
      </div>
      <div class="exif-item" v-if="exif.datetime">
        <span class="exif-label">拍摄时间</span>
        <span class="exif-value">{{ exif.datetime }}</span>
      </div>
      <div class="exif-item" v-if="exif.gpsLatitude != null && exif.gpsLongitude != null">
        <span class="exif-label">拍摄地点</span>
        <span class="exif-value">{{ formatGps(exif.gpsLatitude, exif.gpsLongitude) }}</span>
      </div>
      <div class="exif-item" v-if="exif.cameraMake || exif.cameraModel">
        <span class="exif-label">相机</span>
        <span class="exif-value">{{ [exif.cameraMake, exif.cameraModel].filter(Boolean).join(' ') }}</span>
      </div>
      <div class="exif-item" v-if="exif.lensModel">
        <span class="exif-label">镜头</span>
        <span class="exif-value">{{ exif.lensModel }}</span>
      </div>
      <div class="exif-item" v-if="exif.focalLength != null">
        <span class="exif-label">焦距</span>
        <span class="exif-value">{{ exif.focalLength }}mm</span>
      </div>
      <div class="exif-item" v-if="exif.shutterSpeed">
        <span class="exif-label">快门</span>
        <span class="exif-value">{{ exif.shutterSpeed }}</span>
      </div>
      <div class="exif-item" v-if="exif.aperture != null">
        <span class="exif-label">光圈</span>
        <span class="exif-value">f/{{ exif.aperture }}</span>
      </div>
      <div class="exif-item" v-if="exif.iso != null">
        <span class="exif-label">ISO</span>
        <span class="exif-value">{{ exif.iso }}</span>
      </div>
      <div class="exif-item" v-if="exif.imageWidth != null && exif.imageHeight != null">
        <span class="exif-label">尺寸</span>
        <span class="exif-value">{{ exif.imageWidth }} x {{ exif.imageHeight }}</span>
      </div>
    </div>
    <div v-else class="exif-empty">无 EXIF 信息</div>

    <!-- 评级控制 -->
    <div class="rating-control">
      <h3 class="section-title">评级</h3>
      <div v-if="!image" class="rating-empty">未选择图像</div>
      <div v-else class="stars">
        <span
          v-for="i in 5"
          :key="i"
          class="star"
          :class="{ active: i <= (exif?.rating || 0) }"
          @click="setRating(i === (exif?.rating || 0) ? 0 : i)"
        >
          &#9733;
        </span>
        <span class="rating-label">{{ (exif?.rating || 0) > 0 ? (exif?.rating || 0) + ' 星' : '未评级' }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.exif-display {
  margin-bottom: var(--spacing-lg);
}

.section-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  margin-bottom: var(--spacing-md);
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border-light);
}

.exif-empty,
.exif-loading {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.exif-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.exif-item {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: var(--spacing-sm);
}

.exif-label {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.exif-value {
  font-size: var(--font-size-sm);
  color: var(--color-text-primary);
  text-align: right;
  word-break: break-all;
}

/* 评级控制样式 */
.rating-control {
  margin-top: var(--spacing-lg);
}

.rating-empty {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.stars {
  display: flex;
  align-items: center;
  gap: 4px;
}

.star {
  font-size: 22px;
  cursor: pointer;
  color: var(--color-star-empty);
  transition: color var(--transition-fast);
}

.star.active {
  color: var(--color-star);
}

.star:hover {
  color: var(--color-star);
}

.rating-label {
  margin-left: var(--spacing-sm);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}
</style>
