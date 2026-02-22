<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ImageGroup } from '../../types/image'

const props = defineProps<{
  image: ImageGroup | null
  format: 'jpg' | 'raw'
}>()

const imageData = ref<string | null>(null)
const loading = ref(false)

async function loadImage() {
  if (!props.image) {
    imageData.value = null
    return
  }
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
  <div class="image-viewer">
    <div v-if="!image" class="viewer-empty">选择图像以预览</div>
    <div v-else-if="loading" class="viewer-loading">加载中...</div>
    <div v-else-if="imageData" class="viewer-content">
      <img
        :src="'data:image/jpeg;base64,' + imageData"
        class="viewer-img"
        draggable="false"
      />
    </div>
    <div v-else class="viewer-empty">无法加载图像</div>
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
}

.viewer-empty,
.viewer-loading {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.viewer-content {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-md);
}

.viewer-img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
</style>
