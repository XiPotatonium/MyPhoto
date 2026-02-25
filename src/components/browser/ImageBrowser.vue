<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ImageGroup, SortField, SortOrder } from '../../types/image'
import ImageThumbnail from './ImageThumbnail.vue'
import ContextMenu from '../common/ContextMenu.vue'
import { useContextMenu } from '../../composables/useContextMenu'
import { RecycleScroller } from 'vue-virtual-scroller'

interface ThumbnailResult {
  base_name: string
  thumbnail: string | null
  error: string | null
}

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
const containerWidth = ref(0)

// 计算每行显示的缩略图数量
const itemsPerRow = computed(() => {
  const thumbnailSize = 150 // 默认缩略图大小
  const gap = 8 // 默认间距
  const padding = 16 // 默认内边距
  const availableWidth = containerWidth.value - padding * 2
  if (availableWidth <= 0) return 4 // 默认值
  return Math.max(1, Math.floor((availableWidth + gap) / (thumbnailSize + gap)))
})

// 将图片分组成行
const imageRows = computed(() => {
  const rows: Array<{ index: number; images: Array<{ img: ImageGroup; globalIndex: number }> }> = []
  for (let i = 0; i < images.value.length; i += itemsPerRow.value) {
    rows.push({
      index: i,
      images: images.value.slice(i, i + itemsPerRow.value).map((img, offset) => ({
        img,
        globalIndex: i + offset
      }))
    })
  }
  return rows
})

// 计算行高度
const rowHeight = 150 + 8 // 缩略图高度 + 间距

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
    loadThumbnailsBatch() // 不使用 await，让它在后台运行
  } catch (e) {
    console.error('Failed to list images:', e)
    images.value = []
  } finally {
    loading.value = false
  }
}

async function loadThumbnailsBatch() {
  if (images.value.length === 0) return
  
  // 设置事件监听器
  const unlisten = await listen<ThumbnailResult>('thumbnail-ready', (event) => {
    const { base_name, thumbnail, error } = event.payload
    if (thumbnail) {
      thumbnails.value.set(base_name, thumbnail)
    } else if (error) {
      console.error(`Failed to generate thumbnail for ${base_name}:`, error)
    }
  })
  
  // 收集所有文件路径
  const filePaths = images.value
    .map(img => img.jpgPath || img.rawPath)
    .filter(path => path !== null) as string[]
  
  try {
    // 调用批量生成命令
    await invoke('generate_thumbnails_batch', { filePaths })
  } catch (e) {
    console.error('Failed to generate thumbnails batch:', e)
  } finally {
    // 清理事件监听器
    unlisten()
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

// 监听容器宽度变化
const browserEl = ref<HTMLElement | null>(null)
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (browserEl.value) {
    containerWidth.value = browserEl.value.offsetWidth
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerWidth.value = entry.contentRect.width
      }
    })
    resizeObserver.observe(browserEl.value)
  }
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})

defineExpose({ navigateImage, requestDelete, selectedIndices, images })
</script>

<template>
  <div ref="browserEl" class="image-browser" @contextmenu.prevent="onContextMenu">
    <div v-if="!selectedFolder" class="browser-empty">选择一个文件夹以浏览图像</div>
    <div v-else-if="loading" class="browser-loading">加载中...</div>
    <div v-else-if="images.length === 0" class="browser-empty">此文件夹中没有图像文件</div>
    <RecycleScroller
      v-else
      class="scroller"
      :items="imageRows"
      :item-size="rowHeight"
      key-field="index"
      v-slot="{ item }"
    >
      <div class="thumbnail-row">
        <ImageThumbnail
          v-for="imgData in item.images"
          :key="imgData.img.baseName"
          :image="imgData.img"
          :thumbnail="thumbnails.get(imgData.img.baseName) || null"
          :selected="selectedIndices.has(imgData.globalIndex)"
          @click="(e: MouseEvent) => onImageClick(imgData.globalIndex, e)"
        />
      </div>
    </RecycleScroller>
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
  display: flex;
  flex-direction: column;
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

.scroller {
  flex: 1;
  height: 100%;
}

.thumbnail-row {
  display: flex;
  gap: var(--spacing-sm);
  padding: 0 var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
}

/* 保留原 grid 样式作为 fallback */
.thumbnail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--thumbnail-size), 1fr));
  gap: var(--spacing-sm);
}
</style>
