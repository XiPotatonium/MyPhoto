<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ImageOff, Loader2, ZoomIn, ZoomOut } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { ButtonGroup } from '@/components/ui/button-group'
import type { ImageGroup } from '../../types/image'

const props = defineProps<{
  image: ImageGroup | null
  format: 'jpg' | 'raw'
}>()

const viewportRef = ref<HTMLDivElement>()
const imgRef = ref<HTMLImageElement>()

const imageData = ref<string | null>(null)
const loading = ref(false)

// 图片固有尺寸（原始像素）
const naturalWidth = ref(0)
const naturalHeight = ref(0)
// 视口尺寸
const viewportWidth = ref(0)
const viewportHeight = ref(0)

// 相对于原始像素的缩放比例（1.0 = 1:1 原始像素）
const pixelScale = ref(1)
const MAX_PIXEL_SCALE = 2
const ZOOM_STEP = 0.1

const translateX = ref(0)
const translateY = ref(0)
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const dragStartTranslateX = ref(0)
const dragStartTranslateY = ref(0)

// 适应窗口的缩放比例
const fitScale = computed(() => {
  if (naturalWidth.value === 0 || viewportWidth.value === 0) return 1
  const scaleX = viewportWidth.value / naturalWidth.value
  const scaleY = viewportHeight.value / naturalHeight.value
  return Math.min(scaleX, scaleY)
})

const minPixelScale = computed(() => fitScale.value)

const scalePercent = computed(() => Math.round(pixelScale.value * 100) + '%')
const canZoomIn = computed(() => pixelScale.value < MAX_PIXEL_SCALE)
const canZoomOut = computed(() => pixelScale.value > minPixelScale.value)

// 是否可以拖拽：图片实际尺寸超过视口时才可拖拽
const isPanEnabled = computed(() => {
  if (naturalWidth.value === 0 || viewportWidth.value === 0) return false
  const imgW = naturalWidth.value * pixelScale.value
  const imgH = naturalHeight.value * pixelScale.value
  return imgW > viewportWidth.value || imgH > viewportHeight.value
})

const isZoomed = computed(() => Math.abs(pixelScale.value - fitScale.value) > 0.001)

function updateViewportSize() {
  if (viewportRef.value) {
    viewportWidth.value = viewportRef.value.clientWidth
    viewportHeight.value = viewportRef.value.clientHeight
  }
}

function onImageLoad() {
  if (imgRef.value) {
    naturalWidth.value = imgRef.value.naturalWidth
    naturalHeight.value = imgRef.value.naturalHeight
    updateViewportSize()
    // 初始状态：适应窗口
    pixelScale.value = fitScale.value
    translateX.value = 0
    translateY.value = 0
  }
}

function resetTransform() {
  // 恢复至初始状态：适应窗口
  pixelScale.value = fitScale.value
  translateX.value = 0
  translateY.value = 0
}

function zoomIn() {
  if (!canZoomIn.value) return
  pixelScale.value = Math.min(pixelScale.value + ZOOM_STEP, MAX_PIXEL_SCALE)
}

function zoomOut() {
  if (!canZoomOut.value) return
  pixelScale.value = Math.max(pixelScale.value - ZOOM_STEP, minPixelScale.value)
}

function onWheel(e: WheelEvent) {
  e.preventDefault()
  const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP
  pixelScale.value = Math.min(
    Math.max(pixelScale.value + delta, minPixelScale.value),
    MAX_PIXEL_SCALE
  )
}

function onMouseDown(e: MouseEvent) {
  if (!isPanEnabled.value) return
  isDragging.value = true
  dragStartX.value = e.clientX
  dragStartY.value = e.clientY
  dragStartTranslateX.value = translateX.value
  dragStartTranslateY.value = translateY.value
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  const dx = e.clientX - dragStartX.value
  const dy = e.clientY - dragStartY.value
  translateX.value = dragStartTranslateX.value + dx
  translateY.value = dragStartTranslateY.value + dy
}

function onMouseUp() {
  isDragging.value = false
}

function onMouseLeave() {
  isDragging.value = false
}

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (viewportRef.value) {
    updateViewportSize()
    resizeObserver = new ResizeObserver(() => {
      updateViewportSize()
    })
    resizeObserver.observe(viewportRef.value)
  }
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})

async function loadImage() {
  if (!props.image) {
    imageData.value = null
    naturalWidth.value = 0
    naturalHeight.value = 0
    pixelScale.value = 1
    translateX.value = 0
    translateY.value = 0
    return
  }
  // 重置状态，等待新图片加载完成后在 onImageLoad 中设置 fitScale
  naturalWidth.value = 0
  naturalHeight.value = 0
  pixelScale.value = 1
  translateX.value = 0
  translateY.value = 0

  const filePath = props.format === 'jpg' ? props.image.jpgPath : props.image.rawPath
  if (!filePath) {
    const fallback = props.image.jpgPath || props.image.rawPath
    if (!fallback) { imageData.value = null; return }
    loading.value = true
    try {
      imageData.value = await invoke<string>('read_image_file', { filePath: fallback })
    } catch (e) {
      console.error('Failed to load image:', e)
      imageData.value = null
    } finally {
      loading.value = false
    }
    return
  }
  loading.value = true
  try {
    imageData.value = await invoke<string>('read_image_file', { filePath })
  } catch (e) {
    console.error('Failed to load image:', e)
    imageData.value = null
  } finally {
    loading.value = false
  }
}

watch(() => [props.image, props.format], loadImage, { immediate: true })
</script>

<template>
  <div
    class="image-viewer"
    @contextmenu.prevent
    @wheel.prevent="onWheel"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @mouseleave="onMouseLeave"
  >
    <!-- 缩放控件 -->
    <div v-if="image && !loading && imageData" class="viewer-zoom-controls">
      <ButtonGroup class="viewer-zoom-controls-group">
        <Button
          variant="ghost"
          size="icon-sm"
          :disabled="!canZoomOut"
          @click="zoomOut"
          title="缩小"
        >
          <ZoomOut class="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          class="viewer-scale-btn"
          :class="{ 'viewer-scale-active': isZoomed }"
          @click="resetTransform"
          title="恢复初始状态"
        >
          <span class="viewer-scale-text">{{ scalePercent }}</span>
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          :disabled="!canZoomIn"
          @click="zoomIn"
          title="放大"
        >
          <ZoomIn class="h-4 w-4" />
        </Button>
      </ButtonGroup>
    </div>

    <div ref="viewportRef" class="viewer-viewport">
      <template v-if="!image">
        <div class="viewer-empty-state">
          <ImageOff class="h-16 w-16 text-muted-foreground/30" />
          <p>选择图像以预览</p>
        </div>
      </template>
      <template v-else-if="loading">
        <div class="viewer-loading">
          <Loader2 class="h-10 w-10 animate-spin text-muted-foreground" />
          <span>加载图像...</span>
        </div>
      </template>
      <template v-else-if="imageData">
        <img
          ref="imgRef"
          :src="'data:image/jpeg;base64,' + imageData"
          class="viewer-img"
          draggable="false"
          :style="{
            width: naturalWidth > 0 ? `${naturalWidth * pixelScale}px` : 'auto',
            aspectRatio: naturalWidth > 0 && naturalHeight > 0 ? `${naturalWidth} / ${naturalHeight}` : 'auto',
            maxWidth: 'none',
            maxHeight: 'none',
            transform: `translate(${translateX}px, ${translateY}px)`,
            transition: isDragging ? 'none' : 'transform 150ms ease-out',
            cursor: isPanEnabled ? (isDragging ? 'grabbing' : 'grab') : 'default'
          }"
          @load="onImageLoad"
        />
      </template>
      <template v-else>
        <div class="viewer-empty-state">
          <ImageOff class="h-16 w-16 text-muted-foreground/30" />
          <p>无法加载图像</p>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.image-viewer {
  position: relative;
  flex: 1;
  overflow: hidden;
  background: var(--background);
  user-select: none;
}

.viewer-zoom-controls {
  position: absolute;
  top: var(--spacing-4);
  right: var(--spacing-4);
  z-index: var(--z-docked);
  opacity: 0.85;
  transition: opacity var(--transition-fast);
}

.viewer-zoom-controls:hover {
  opacity: 1;
}

.viewer-zoom-controls-group {
  background: var(--card);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-md);
  border-radius: var(--radius);
  padding: var(--spacing-1);
  gap: var(--spacing-0-5);
}

.viewer-scale-btn {
  min-width: 3.5rem;
  font-variant-numeric: tabular-nums;
}

.viewer-scale-text {
  font-size: var(--font-size-xs);
  font-weight: var(--font-weight-medium);
  color: var(--muted-foreground);
}

.viewer-scale-active .viewer-scale-text {
  color: var(--primary);
}

.viewer-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
}

.viewer-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
}

.viewer-viewport {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.viewer-img {
  display: block;
  box-shadow: var(--shadow-lg);
  flex-shrink: 0;
  transform-origin: center center;
  will-change: transform;
}
</style>
