<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ImageGroup } from '../../types/image'

const props = defineProps<{
  image: ImageGroup | null
  format: 'jpg' | 'raw'
}>()

const imageData = ref<string | null>(null)
const loading = ref(false)
const imageLoaded = ref(false)
const prevImageData = ref<string | null>(null)

const displayImage = computed(() => {
  return imageData.value || prevImageData.value
})

async function loadImage() {
  if (!props.image) {
    imageData.value = null
    prevImageData.value = null
    imageLoaded.value = false
    return
  }
  
  // Keep previous image visible during loading for smooth transition
  if (imageData.value) {
    prevImageData.value = imageData.value
  }
  
  imageLoaded.value = false
  const filePath = props.format === 'jpg' ? props.image.jpgPath : props.image.rawPath
  if (!filePath) {
    const fallback = props.image.jpgPath || props.image.rawPath
    if (!fallback) { 
      imageData.value = null
      prevImageData.value = null
      return 
    }
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

function onImageLoad() {
  imageLoaded.value = true
  prevImageData.value = null
}

watch(() => [props.image, props.format], loadImage, { immediate: true })
</script>

<template>
  <div class="image-viewer">
    <!-- Empty State -->
    <div v-if="!image" class="viewer-state empty">
      <div class="state-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="3" width="18" height="18" rx="2"/>
          <circle cx="8.5" cy="8.5" r="1.5"/>
          <path d="M21 15l-5-5L5 21"/>
        </svg>
      </div>
      <div class="state-title">选择图像</div>
      <div class="state-desc">从左侧选择一张图片开始预览</div>
    </div>
    
    <!-- Loading State -->
    <div v-else-if="loading && !displayImage" class="viewer-state loading">
      <div class="loading-spinner">
        <div class="spinner-ring"></div>
        <div class="spinner-ring"></div>
        <div class="spinner-ring"></div>
      </div>
      <div class="loading-text">加载图像中...</div>
    </div>
    
    <!-- Image Display -->
    <div v-else-if="displayImage" class="viewer-content">
      <!-- Previous image (for crossfade) -->
      <img
        v-if="prevImageData && !imageLoaded"
        :src="'data:image/jpeg;base64,' + prevImageData"
        class="viewer-img prev"
        draggable="false"
      />
      
      <!-- Current image -->
      <img
        :src="'data:image/jpeg;base64,' + displayImage"
        class="viewer-img"
        :class="{ loaded: imageLoaded }"
        draggable="false"
        @load="onImageLoad"
      />
      
      <!-- Loading indicator overlay -->
      <div v-if="loading" class="loading-overlay">
        <div class="loading-dots">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>
    </div>
    
    <!-- Error State -->
    <div v-else class="viewer-state error">
      <div class="state-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 8v4M12 16h.01"/>
        </svg>
      </div>
      <div class="state-title">无法加载图像</div>
      <div class="state-desc">文件可能已损坏或格式不受支持</div>
    </div>
  </div>
</template>

<style scoped>
.image-viewer {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: var(--color-bg-primary);
  position: relative;
}

/* State Screens (Empty, Loading, Error) */
.viewer-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: var(--spacing-8);
  animation: fadeIn var(--transition-slow) ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.state-icon {
  width: 64px;
  height: 64px;
  color: var(--color-text-muted);
  margin-bottom: var(--spacing-4);
  opacity: 0.5;
}

.state-icon svg {
  width: 100%;
  height: 100%;
}

.state-title {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-1);
}

.state-desc {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

/* Loading Animation */
.loading-spinner {
  position: relative;
  width: 56px;
  height: 56px;
  margin-bottom: var(--spacing-4);
}

.spinner-ring {
  position: absolute;
  inset: 0;
  border: 3px solid transparent;
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 1.2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
}

.spinner-ring:nth-child(1) {
  animation-delay: -0.45s;
}

.spinner-ring:nth-child(2) {
  animation-delay: -0.3s;
  border-top-color: var(--color-accent);
  opacity: 0.6;
}

.spinner-ring:nth-child(3) {
  animation-delay: -0.15s;
  border-top-color: var(--color-accent);
  opacity: 0.3;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-text {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}

/* Viewer Content */
.viewer-content {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-4);
  position: relative;
}

.viewer-img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  opacity: 0;
  transform: scale(0.98);
  transition: all var(--transition-slow) var(--ease-out-expo);
}

.viewer-img.loaded {
  opacity: 1;
  transform: scale(1);
}

.viewer-img.prev {
  position: absolute;
  opacity: 0.5;
  transform: scale(0.98);
  filter: blur(2px);
}

/* Loading Overlay */
.loading-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-scrim);
  backdrop-filter: blur(2px);
  border-radius: var(--radius-md);
  animation: fadeIn var(--transition-fast);
}

.loading-dots {
  display: flex;
  gap: 6px;
}

.loading-dots span {
  width: 8px;
  height: 8px;
  background: white;
  border-radius: 50%;
  animation: bounce 1.4s ease-in-out infinite both;
}

.loading-dots span:nth-child(1) {
  animation-delay: -0.32s;
}

.loading-dots span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%, 80%, 100% { 
    transform: scale(0);
    opacity: 0.5;
  }
  40% { 
    transform: scale(1);
    opacity: 1;
  }
}

/* Error State */
.viewer-state.error .state-icon {
  color: var(--color-danger);
  opacity: 0.7;
}

.viewer-state.error .state-title {
  color: var(--color-danger);
}
</style>
