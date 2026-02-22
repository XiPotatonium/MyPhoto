<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ImageGroup, SortField, SortOrder } from '../../types/image'
import ImageThumbnail from './ImageThumbnail.vue'
import ContextMenu from '../common/ContextMenu.vue'
import { useContextMenu } from '../../composables/useContextMenu'

const props = defineProps<{
  selectedFolder: string | null
  sortField: SortField
  sortOrder: SortOrder
}>()

const emit = defineEmits<{
  'image-selected': [image: ImageGroup]
}>()

const images = ref<ImageGroup[]>([])
const loading = ref(false)
const selectedIndex = ref(-1)
const selectedIndices = ref<Set<number>>(new Set())
const thumbnails = ref<Map<string, string>>(new Map())
const { menuState, showMenu, hideMenu } = useContextMenu()

async function loadImages() {
  if (!props.selectedFolder) {
    images.value = []
    return
  }
  loading.value = true
  try {
    images.value = await invoke<ImageGroup[]>('list_images', {
      dirPath: props.selectedFolder,
      sortField: props.sortField,
      sortOrder: props.sortOrder,
    })
    selectedIndex.value = -1
    selectedIndices.value.clear()
    thumbnails.value.clear()
    loadThumbnailsBatch()
  } catch (e) {
    console.error('Failed to list images:', e)
    images.value = []
  } finally {
    loading.value = false
  }
}

async function loadThumbnailsBatch() {
  for (const img of images.value) {
    const filePath = img.jpgPath || img.rawPath
    if (!filePath) continue
    try {
      const thumb = await invoke<string>('generate_thumbnail', { filePath })
      thumbnails.value.set(img.baseName, thumb)
    } catch (e) {
      console.error('Failed to generate thumbnail:', filePath, e)
    }
  }
}

function onImageClick(index: number, e: MouseEvent) {
  if (e.shiftKey && selectedIndex.value >= 0) {
    const start = Math.min(selectedIndex.value, index)
    const end = Math.max(selectedIndex.value, index)
    selectedIndices.value.clear()
    for (let i = start; i <= end; i++) {
      selectedIndices.value.add(i)
    }
  } else {
    selectedIndex.value = index
    selectedIndices.value.clear()
    selectedIndices.value.add(index)
  }
  emit('image-selected', images.value[index])
}

function onContextMenu(e: MouseEvent) {
  showMenu(e, [
    { label: '刷新文件夹', action: loadImages },
    { label: '删除', action: requestDelete },
  ])
}

function requestDelete() {
  // Will be wired up in Phase 9
}

function navigateImage(direction: number) {
  if (images.value.length === 0) return
  let newIndex = selectedIndex.value + direction
  if (newIndex < 0) newIndex = 0
  if (newIndex >= images.value.length) newIndex = images.value.length - 1
  selectedIndex.value = newIndex
  selectedIndices.value.clear()
  selectedIndices.value.add(newIndex)
  emit('image-selected', images.value[newIndex])
}

watch(() => props.selectedFolder, loadImages)
watch(() => [props.sortField, props.sortOrder], loadImages)

defineExpose({ navigateImage, requestDelete, selectedIndices, images })
</script>

<template>
  <div class="image-browser" @contextmenu.prevent="onContextMenu">
    <div v-if="!selectedFolder" class="browser-empty">选择一个文件夹以浏览图像</div>
    <div v-else-if="loading" class="browser-loading">加载中...</div>
    <div v-else-if="images.length === 0" class="browser-empty">此文件夹中没有图像文件</div>
    <div v-else class="thumbnail-grid">
      <ImageThumbnail
        v-for="(img, i) in images"
        :key="img.baseName"
        :image="img"
        :thumbnail="thumbnails.get(img.baseName) || null"
        :selected="selectedIndices.has(i)"
        @click="(e: MouseEvent) => onImageClick(i, e)"
      />
    </div>
    <ContextMenu
      :visible="menuState.visible"
      :x="menuState.x"
      :y="menuState.y"
      :items="menuState.items"
      @close="hideMenu"
    />
  </div>
</template>

<style scoped>
.image-browser {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  padding: var(--spacing-sm);
}

.browser-empty,
.browser-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.thumbnail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--thumbnail-size), 1fr));
  gap: var(--spacing-sm);
}
</style>
