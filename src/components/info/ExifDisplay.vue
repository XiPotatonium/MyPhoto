<script setup lang="ts">
import { computed } from 'vue'
import { Camera, Clock, MapPin, Aperture, Gauge, Focus, Ruler, Image as ImageIcon, Star, RotateCw, Calendar } from 'lucide-vue-next'
import type { ExifInfo } from '../../types/exif'
import { formatGps, formatDateTime, formatShutterSpeed } from '../../utils/formatters'
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card'
import { useAppState } from '../../stores/appState'

const appState = useAppState()

const image = computed(() => appState.state.currentImage)
const exif = computed<ExifInfo | null>(() => {
  const img = image.value
  return img?.exifInfo ?? null
})

function orientationLabel(value: number): string {
  switch (value) {
    case 1: return '正常'
    case 3: return '旋转180°'
    case 6: return '顺时针90°'
    case 8: return '逆时针90°'
    default: return '未知'
  }
}

</script>

<template>
  <Card class="info-card exif-display">
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
      <div v-else-if="exif" class="exif-list">
        <div v-if="image.baseName" class="exif-item">
          <span class="exif-label">文件名</span>
          <span class="exif-value">{{ image.baseName }}</span>
        </div>
        <div v-if="exif.datetime" class="exif-item">
          <Clock class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">拍摄时间</span>
          <span class="exif-value">{{ formatDateTime(exif.datetime) }}</span>
        </div>
        <div v-if="exif.fileCreatedTime" class="exif-item">
          <Calendar class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">创建时间</span>
          <span class="exif-value">{{ formatDateTime(exif.fileCreatedTime) }}</span>
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
          <span class="exif-value">{{ formatShutterSpeed(exif.shutterSpeed) }}</span>
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
        <div v-if="exif.rating != null && exif.rating > 0" class="exif-item">
          <Star class="exif-icon h-3.5 w-3.5 fill-current" />
          <span class="exif-label">评分</span>
          <span class="exif-value">{{ exif.rating }} / 5</span>
        </div>
        <div v-if="exif.imageWidth != null && exif.imageHeight != null" class="exif-item">
          <ImageIcon class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">尺寸</span>
          <span class="exif-value">{{ exif.imageWidth }} × {{ exif.imageHeight }}</span>
        </div>
        <div v-if="exif.orientation != null" class="exif-item">
          <RotateCw class="exif-icon h-3.5 w-3.5" />
          <span class="exif-label">方向</span>
          <span class="exif-value">{{ exif.orientation }} ({{ orientationLabel(exif.orientation) }})</span>
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
  /* Let Card defaults (rounded-xl, border, shadow-sm) create floating card effect */
  padding-block: 0 !important;
  gap: 0 !important;
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
  border-radius: var(--radius-sm);
  margin: 0 calc(-1 * var(--spacing-xs));
  padding-left: var(--spacing-xs);
  padding-right: var(--spacing-xs);
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
  font-weight: 500;
  user-select: text;
}

</style>
