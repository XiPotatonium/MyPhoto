<script setup lang="ts">
import { ref, watch, computed } from 'vue'
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

// Group EXIF data into categories
const cameraInfo = computed(() => {
  if (!exif.value) return []
  const items = []
  if (exif.value.cameraMake || exif.value.cameraModel) {
    items.push({
      label: '相机',
      value: [exif.value.cameraMake, exif.value.cameraModel].filter(Boolean).join(' '),
      icon: '📷'
    })
  }
  if (exif.value.lensModel) {
    items.push({ label: '镜头', value: exif.value.lensModel, icon: '🔍' })
  }
  return items
})

const exposureInfo = computed(() => {
  if (!exif.value) return []
  const items = []
  if (exif.value.focalLength != null) {
    items.push({ label: '焦距', value: `${exif.value.focalLength}mm`, icon: '📐' })
  }
  if (exif.value.aperture != null) {
    items.push({ label: '光圈', value: `f/${exif.value.aperture}`, icon: '🔅' })
  }
  if (exif.value.shutterSpeed) {
    items.push({ label: '快门', value: exif.value.shutterSpeed, icon: '⏱️' })
  }
  if (exif.value.iso != null) {
    items.push({ label: 'ISO', value: exif.value.iso.toString(), icon: '📊' })
  }
  return items
})

const fileInfo = computed(() => {
  if (!exif.value || !props.image) return []
  const items = []
  if (props.image.baseName) {
    items.push({ label: '文件名', value: props.image.baseName, icon: '📄' })
  }
  if (exif.value.datetime) {
    items.push({ label: '拍摄时间', value: exif.value.datetime, icon: '📅' })
  }
  if (exif.value.gpsLatitude != null && exif.value.gpsLongitude != null) {
    items.push({ 
      label: 'GPS', 
      value: formatGps(exif.value.gpsLatitude, exif.value.gpsLongitude),
      icon: '📍'
    })
  }
  if (exif.value.imageWidth != null && exif.value.imageHeight != null) {
    const mp = ((exif.value.imageWidth * exif.value.imageHeight) / 1000000).toFixed(1)
    items.push({ 
      label: '尺寸', 
      value: `${exif.value.imageWidth} × ${exif.value.imageHeight} (${mp}MP)`,
      icon: '🖼️'
    })
  }
  return items
})

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
    <!-- Empty State -->
    <div v-if="!image" class="state-panel empty">
      <div class="state-icon">🖼️</div>
      <div class="state-text">选择图像查看详细信息</div>
    </div>
    
    <!-- Loading State -->
    <div v-else-if="loading && !exif" class="state-panel loading">
      <div class="loading-ring"></div>
      <div class="state-text">读取 EXIF 信息...</div>
    </div>
    
    <!-- EXIF Content -->
    <template v-else>
      <!-- Rating Card -->
      <div class="info-card rating-card">
        <div class="card-header">
          <span class="header-icon">⭐</span>
          <span class="header-title">评分</span>
        </div>
        <div class="rating-content">
          <div class="stars">
            <button
              v-for="i in 5"
              :key="i"
              class="star-btn"
              :class="{ active: i <= (exif?.rating || 0) }"
              @click="setRating(i === (exif?.rating || 0) ? 0 : i)"
              :title="`${i} 星`"
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
              </svg>
            </button>
          </div>
          <div class="rating-text">
            {{ (exif?.rating || 0) > 0 ? `${exif?.rating} 星评分` : '点击星星评分' }}
          </div>
        </div>
      </div>
      
      <!-- File Info Card -->
      <div class="info-card" v-if="fileInfo.length > 0">
        <div class="card-header">
          <span class="header-icon">📋</span>
          <span class="header-title">文件信息</span>
        </div>
        <div class="info-list">
          <div v-for="item in fileInfo" :key="item.label" class="info-item">
            <span class="item-icon">{{ item.icon }}</span>
            <span class="item-label">{{ item.label }}</span>
            <span class="item-value" :title="item.value">{{ item.value }}</span>
          </div>
        </div>
      </div>
      
      <!-- Camera Info Card -->
      <div class="info-card" v-if="cameraInfo.length > 0">
        <div class="card-header">
          <span class="header-icon">📷</span>
          <span class="header-title">相机设备</span>
        </div>
        <div class="info-list">
          <div v-for="item in cameraInfo" :key="item.label" class="info-item">
            <span class="item-icon">{{ item.icon }}</span>
            <span class="item-label">{{ item.label }}</span>
            <span class="item-value" :title="item.value">{{ item.value }}</span>
          </div>
        </div>
      </div>
      
      <!-- Exposure Info Card -->
      <div class="info-card" v-if="exposureInfo.length > 0">
        <div class="card-header">
          <span class="header-icon">⚙️</span>
          <span class="header-title">曝光参数</span>
        </div>
        <div class="info-grid">
          <div v-for="item in exposureInfo" :key="item.label" class="exposure-item">
            <span class="exposure-icon">{{ item.icon }}</span>
            <span class="exposure-value">{{ item.value }}</span>
            <span class="exposure-label">{{ item.label }}</span>
          </div>
        </div>
      </div>
      
      <!-- No EXIF State -->
      <div v-if="!exif && !loading" class="state-panel no-data">
        <div class="state-icon">📭</div>
        <div class="state-text">无 EXIF 信息</div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.exif-display {
  padding: var(--spacing-3);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

/* State Panels */
.state-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-8) var(--spacing-4);
  text-align: center;
  color: var(--color-text-muted);
}

.state-icon {
  font-size: 48px;
  margin-bottom: var(--spacing-3);
  opacity: 0.5;
}

.state-text {
  font-size: var(--font-size-sm);
}

.loading-ring {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: var(--spacing-3);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Info Cards */
.info-card {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-xs);
  transition: all var(--transition-fast);
}

.info-card:hover {
  box-shadow: var(--shadow-sm);
  border-color: var(--color-border-strong);
}

.card-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-3) var(--spacing-4);
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
}

.header-icon {
  font-size: var(--font-size-md);
}

.header-title {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-primary);
}

/* Rating Card */
.rating-card {
  background: linear-gradient(135deg, var(--color-accent-alpha) 0%, transparent 100%);
}

.rating-content {
  padding: var(--spacing-4);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-2);
}

.stars {
  display: flex;
  gap: var(--spacing-1);
}

.star-btn {
  width: 32px;
  height: 32px;
  padding: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--color-star-empty);
  transition: all var(--transition-fast);
}

.star-btn svg {
  width: 100%;
  height: 100%;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.1));
}

.star-btn:hover {
  transform: scale(1.15);
  color: var(--color-star);
}

.star-btn.active {
  color: var(--color-star);
  animation: starPop 0.3s var(--ease-out-expo);
}

@keyframes starPop {
  0% { transform: scale(1); }
  50% { transform: scale(1.3); }
  100% { transform: scale(1); }
}

.rating-text {
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  font-weight: var(--font-weight-medium);
}

/* Info List */
.info-list {
  padding: var(--spacing-2) 0;
}

.info-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-4);
  transition: background var(--transition-fast);
}

.info-item:hover {
  background: var(--color-bg-hover);
}

.item-icon {
  font-size: var(--font-size-sm);
  width: 20px;
  text-align: center;
  flex-shrink: 0;
}

.item-label {
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  min-width: 56px;
  flex-shrink: 0;
}

.item-value {
  flex: 1;
  font-size: var(--font-size-xs);
  color: var(--color-text-primary);
  text-align: right;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: var(--font-weight-medium);
}

/* Exposure Grid */
.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-1);
  padding: var(--spacing-3);
}

.exposure-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-3) var(--spacing-2);
  background: var(--color-bg-secondary);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.exposure-item:hover {
  background: var(--color-bg-hover);
  transform: translateY(-1px);
}

.exposure-icon {
  font-size: var(--font-size-lg);
  margin-bottom: var(--spacing-1);
}

.exposure-value {
  font-size: var(--font-size-md);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-primary);
  font-family: var(--font-family-mono);
}

.exposure-label {
  font-size: var(--font-size-2xs);
  color: var(--color-text-muted);
  margin-top: 2px;
}
</style>
