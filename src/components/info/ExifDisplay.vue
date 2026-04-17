<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Camera, Clock, MapPin, Aperture, Gauge, Focus, Ruler, Image as ImageIcon, Loader2 } from 'lucide-vue-next'
import type { ImageGroup } from '../../types/image'
import type { ExifInfo } from '../../types/exif'
import { formatGps } from '../../utils/formatters'
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card'

const props = defineProps<{
  image: ImageGroup | null
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

watch(() => props.image, loadExif, { immediate: true })
</script>

<template>
  <Card class="info-card">
    <CardHeader class="card-header">
      <CardTitle class="card-title flex items-center gap-2">
        <Camera class="h-4 w-4 text-foreground" />
        图像信息
      </CardTitle>
    </CardHeader>
    <CardContent class="card-content">
      <div v-if="!image" class="empty-state">
        <ImageIcon class="h-10 w-10 text-muted-foreground/30" />
        <p>未选择图像</p>
      </div>
      <div v-else-if="loading" class="loading-state">
        <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
        <span>读取 EXIF 信息...</span>
      </div>
      <div v-else-if="exif" class="exif-list">
        <div v-if="image.baseName" class="exif-item">
          <span class="exif-label">文件名</span>
          <span class="exif-value">{{ image.baseName }}</span>
        </div>
        <div v-if="exif.datetime" class="exif-item">
          <Clock class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">拍摄时间</span>
          <span class="exif-value">{{ exif.datetime }}</span>
        </div>
        <div v-if="exif.gpsLatitude != null && exif.gpsLongitude != null" class="exif-item">
          <MapPin class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">拍摄地点</span>
          <span class="exif-value">{{ formatGps(exif.gpsLatitude, exif.gpsLongitude) }}</span>
        </div>
        <div v-if="exif.cameraMake || exif.cameraModel" class="exif-item">
          <Camera class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">相机</span>
          <span class="exif-value">{{ [exif.cameraMake, exif.cameraModel].filter(Boolean).join(' ') }}</span>
        </div>
        <div v-if="exif.lensModel" class="exif-item">
          <Focus class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">镜头</span>
          <span class="exif-value">{{ exif.lensModel }}</span>
        </div>
        <div v-if="exif.focalLength != null" class="exif-item">
          <Ruler class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">焦距</span>
          <span class="exif-value">{{ exif.focalLength }}mm</span>
        </div>
        <div v-if="exif.shutterSpeed" class="exif-item">
          <Gauge class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">快门</span>
          <span class="exif-value">{{ exif.shutterSpeed }}</span>
        </div>
        <div v-if="exif.aperture != null" class="exif-item">
          <Aperture class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">光圈</span>
          <span class="exif-value">f/{{ exif.aperture }}</span>
        </div>
        <div v-if="exif.iso != null" class="exif-item">
          <Gauge class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">ISO</span>
          <span class="exif-value">{{ exif.iso }}</span>
        </div>
        <div v-if="exif.imageWidth != null && exif.imageHeight != null" class="exif-item">
          <ImageIcon class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">尺寸</span>
          <span class="exif-value">{{ exif.imageWidth }} × {{ exif.imageHeight }}</span>
        </div>
      </div>
      <div v-else class="empty-state">
        <p class="text-muted-foreground text-sm">无 EXIF 信息</p>
      </div>
    </CardContent>
  </Card>
</template>

<style scoped>
.info-card {
  border: 1px solid var(--border);
  border-radius: 0;
  border-left: none;
  border-right: none;
  border-top: none;
}

.card-header {
  padding: var(--spacing-md) var(--spacing-md) var(--spacing-sm);
}

.card-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
}

.card-content {
  padding: var(--spacing-sm) var(--spacing-md) var(--spacing-md);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xl) 0;
  color: var(--muted-foreground);
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xl) 0;
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
}

.exif-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.exif-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) 0;
  border-bottom: 1px solid var(--border);
}

.exif-item:last-child {
  border-bottom: none;
}

.exif-icon {
  color: var(--muted-foreground);
  flex-shrink: 0;
}

.exif-label {
  font-size: var(--font-size-xs);
  color: var(--muted-foreground);
  flex-shrink: 0;
  min-width: 60px;
}

.exif-value {
  font-size: var(--font-size-xs);
  color: var(--foreground);
  text-align: right;
  flex: 1;
  word-break: break-all;
  font-weight: 500;
}

</style>
