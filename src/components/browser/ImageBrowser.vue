<script setup lang="ts">
import { ref, watch, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { FolderOpen, Image, RefreshCw, ChevronUp } from 'lucide-vue-next'
import type { ImageGroup, SortField, SortOrder } from '../../types/image'
import ImageThumbnail from './ImageThumbnail.vue'
import ContextMenu from '../common/ContextMenu.vue'
import ExifDialog from '../common/ExifDialog.vue'
import type { ExifWriteRequest } from '../common/ExifDialog.vue'
import { useContextMenu } from '../../composables/useContextMenu'
import { RecycleScroller } from 'vue-virtual-scroller'

/// 获取图片的排序时间（优先使用 EXIF 时间，其次文件创建时间）
function getSortTime(image: ImageGroup): string | null {
  if (image.exifInfo?.datetime) {
    return image.exifInfo.datetime
  }
  return image.fileCreatedTime
}

/// 获取图片评分
function getRating(image: ImageGroup): number {
  return image.exifInfo?.rating ?? 0
}

/// 前端排序函数
function sortImages(images: ImageGroup[], sortField: SortField, sortOrder: SortOrder): ImageGroup[] {
  const sorted = [...images]

  switch (sortField) {
    case 'name':
      sorted.sort((a, b) => {
        return a.baseName.toLowerCase().localeCompare(b.baseName.toLowerCase())
      })
      break
    case 'date':
      sorted.sort((a, b) => {
        const timeA = getSortTime(a)
        const timeB = getSortTime(b)
        if (!timeA && !timeB) return 0
        if (!timeA) return 1
        if (!timeB) return -1
        return timeA.localeCompare(timeB)
      })
      break
    case 'rating':
      sorted.sort((a, b) => {
        const ratingA = getRating(a)
        const ratingB = getRating(b)
        return ratingA - ratingB
      })
      break
  }

  if (sortOrder === 'desc') {
    sorted.reverse()
  }

  return sorted
}

interface ThumbnailResult {
  base_name: string
  thumbnail: string | null
  error: string | null
}

const props = withDefaults(defineProps<{
  selectedFolder: string | null
  sortField: SortField
  sortOrder: SortOrder
  mode?: 'grid' | 'strip'
}>(), {
  mode: 'grid',
})

const emit = defineEmits<{
  'image-selected': [image: ImageGroup]
  'image-dblclick': [image: ImageGroup]
  'delete-requested': []
  'collapse-up': []
}>()

const rawImages = ref<ImageGroup[]>([])
const loading = ref(false)
const selectedIndex = ref(-1)
const selectedIndices = ref<Set<number>>(new Set())
const thumbnails = ref<Map<string, string>>(new Map())
const { menuState, showMenu, hideMenu } = useContextMenu()
const containerWidth = ref(0)
const gpsDialogVisible = ref(false)
const exifDialogRef = ref<InstanceType<typeof ExifDialog> | null>(null)

// 排序后的图片列表
const images = computed(() => {
  return sortImages(rawImages.value, props.sortField, props.sortOrder)
})

// 当前模式下的缩略图尺寸
const currentThumbnailSize = computed(() => {
  return props.mode === 'grid' ? 260 : 130
})

// 计算每行显示的缩略图数量（仅 grid 模式使用）
const itemsPerRow = computed(() => {
  const thumbnailSize = currentThumbnailSize.value
  const thumbnailPadding = 8
  const itemTotalWidth = thumbnailSize + thumbnailPadding
  const gap = 8
  const padding = 8
  const availableWidth = containerWidth.value - padding * 2

  if (availableWidth <= 0 || containerWidth.value === 0) {
    return 4
  }

  return Math.max(1, Math.floor((availableWidth + gap) / (itemTotalWidth + gap)))
})

// 将图片分组成行（grid 模式）
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
const rowHeight = computed(() => {
  return currentThumbnailSize.value + 24 + 8
})

// Strip 模式下单个缩略图的水平宽度 (thumbnail-size + padding)
const stripItemWidth = 130 + 8

async function loadImages() {
  if (!props.selectedFolder) {
    rawImages.value = []
    return
  }
  loading.value = true
  try {
    rawImages.value = await invoke<ImageGroup[]>('list_images', {
      dirPath: props.selectedFolder,
    })
    selectedIndex.value = -1
    selectedIndices.value.clear()
    thumbnails.value.clear()
    loadThumbnailsBatch()
  } catch (e) {
    console.error('Failed to list images:', e)
    rawImages.value = []
  } finally {
    loading.value = false
  }
}

async function loadThumbnailsBatch() {
  if (images.value.length === 0) return

  const unlisten = await listen<ThumbnailResult>('thumbnail-ready', (event) => {
    const { base_name, thumbnail, error } = event.payload
    if (thumbnail) {
      thumbnails.value.set(base_name, thumbnail)
    } else if (error) {
      console.error(`Failed to generate thumbnail for ${base_name}:`, error)
    }
  })

  const filePaths = images.value
    .map(img => img.jpgPath || img.rawPath)
    .filter(path => path !== null) as string[]

  try {
    await invoke('generate_thumbnails_batch', { filePaths })
  } catch (e) {
    console.error('Failed to generate thumbnails batch:', e)
  } finally {
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

function onImageDblClick(index: number) {
  selectedIndex.value = index
  selectedIndices.value.clear()
  selectedIndices.value.add(index)
  emit('image-dblclick', images.value[index])
}

function onContextMenu(e: MouseEvent) {
  const menuItems = [
    { label: '刷新文件夹', action: loadImages },
    { label: '删除', action: requestDelete },
  ]

  if (selectedIndices.value.size > 0) {
    menuItems.push({ label: '修改EXIF信息', action: openExifDialog })
  }

  showMenu(e, menuItems)
}

function requestDelete() {
  emit('delete-requested')
}

function openExifDialog() {
  if (selectedIndices.value.size === 0) return
  gpsDialogVisible.value = true
}

function closeExifDialog() {
  gpsDialogVisible.value = false
  if (exifDialogRef.value) {
    exifDialogRef.value.reset()
  }
}

async function handleExifConfirm(fields: ExifWriteRequest) {
  if (!exifDialogRef.value) return

  exifDialogRef.value.loading = true

  try {
    const selectedImages = Array.from(selectedIndices.value).map(index => images.value[index])
    const filePaths: string[] = []

    selectedImages.forEach(img => {
      if (img.jpgPath) {
        filePaths.push(img.jpgPath)
      }
      if (img.rawPath) {
        filePaths.push(img.rawPath)
      }
    })

    if (filePaths.length === 0) {
      console.error('No valid file paths found')
      return
    }

    const errors: string[] = []
    for (const filePath of filePaths) {
      try {
        await invoke('write_exif_fields', { filePath, fields })
      } catch (e) {
        errors.push(`${filePath}: ${e}`)
      }
    }

    if (errors.length > 0) {
      console.error('Some files failed:', errors)
      alert('部分文件保存失败:\n' + errors.join('\n'))
    } else {
      console.log(`Successfully updated EXIF for ${filePaths.length} file(s)`)
    }

    closeExifDialog()
  } catch (e) {
    console.error('Failed to write EXIF info:', e)
    alert('保存EXIF信息失败: ' + e)
  } finally {
    if (exifDialogRef.value) {
      exifDialogRef.value.loading = false
    }
  }
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

// 模式切换后恢复滚动位置到选中的图片
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const gridScrollerRef = ref<any>(null)
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const stripScrollerRef = ref<any>(null)

watch(() => props.mode, (newMode) => {
  if (selectedIndex.value >= 0) {
    nextTick(() => {
      if (newMode === 'grid' && gridScrollerRef.value) {
        const rowIndex = Math.floor(selectedIndex.value / itemsPerRow.value)
        gridScrollerRef.value.scrollToItem(rowIndex)
      } else if (newMode === 'strip' && stripScrollerRef.value) {
        stripScrollerRef.value.scrollToItem(selectedIndex.value)
      }
    })
  }
})

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

// 更新当前选中图片的评分
function updateImageRating(rating: number) {
  if (selectedIndex.value < 0 || selectedIndex.value >= images.value.length) return

  const image = images.value[selectedIndex.value]
  if (!image) return

  const rawIndex = rawImages.value.findIndex(img => img.baseName === image.baseName)
  if (rawIndex >= 0 && rawImages.value[rawIndex].exifInfo) {
    rawImages.value[rawIndex].exifInfo!.rating = rating
  }
}

// 前端直接移除图片
function removeImages(paths: string[]) {
  const pathSet = new Set(paths)

  const newRawImages = rawImages.value.filter(img => {
    const jpgDeleted = img.jpgPath && pathSet.has(img.jpgPath)
    const rawDeleted = img.rawPath && pathSet.has(img.rawPath)

    if (jpgDeleted && !rawDeleted && img.rawPath) {
      img.jpgPath = null
      img.fileCount -= 1
      return true
    }
    if (rawDeleted && !jpgDeleted && img.jpgPath) {
      img.rawPath = null
      img.fileCount -= 1
      return true
    }

    return !(jpgDeleted || rawDeleted)
  })

  rawImages.value = newRawImages

  selectedIndices.value.clear()
  selectedIndex.value = -1

  paths.forEach(path => {
    const baseName = path.split('/').pop()?.split('.').shift()
    if (baseName) {
      thumbnails.value.delete(baseName)
    }
  })
}

defineExpose({ navigateImage, requestDelete, selectedIndices, images, updateImageRating, removeImages })
</script>

<template>
  <div
    ref="browserEl"
    class="image-browser"
    @contextmenu.prevent="onContextMenu"
  >
    <!-- Strip 模式：向上收起按钮 -->
    <button
      v-if="mode === 'strip'"
      class="collapse-up-btn"
      title="返回浏览视图"
      @click="emit('collapse-up')"
    >
      <ChevronUp class="h-4 w-4" />
    </button>

    <!-- 空状态 -->
    <div v-if="!selectedFolder" class="browser-empty-state">
      <div class="empty-icon">
        <FolderOpen class="h-12 w-12 text-muted-foreground/50" />
      </div>
      <p class="empty-title">选择文件夹</p>
      <p class="empty-desc">从左侧目录树选择一个文件夹以浏览图像</p>
    </div>
    <div v-else-if="loading" class="browser-loading">
      <RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
      <span>加载图像中...</span>
    </div>
    <div v-else-if="images.length === 0" class="browser-empty-state">
      <div class="empty-icon">
        <Image class="h-12 w-12 text-muted-foreground/50" />
      </div>
      <p class="empty-title">没有图像</p>
      <p class="empty-desc">此文件夹中没有支持的图像文件</p>
    </div>

    <!-- Grid 模式：大图标网格，垂直滚动 -->
    <RecycleScroller
      v-else-if="mode === 'grid'"
      ref="gridScrollerRef"
      class="scroller"
      :items="imageRows"
      :item-size="rowHeight"
      key-field="index"
      direction="vertical"
      v-slot="{ item }"
    >
      <div class="thumbnail-row">
        <ImageThumbnail
          v-for="imgData in item.images"
          :key="imgData.img.baseName"
          :image="imgData.img"
          :thumbnail="thumbnails.get(imgData.img.baseName) || null"
          :selected="selectedIndices.has(imgData.globalIndex)"
          size="large"
          @click="(e: MouseEvent) => onImageClick(imgData.globalIndex, e)"
          @dblclick="() => onImageDblClick(imgData.globalIndex)"
        />
      </div>
    </RecycleScroller>

    <!-- Strip 模式：横向滚动单行 -->
    <RecycleScroller
      v-else
      ref="stripScrollerRef"
      class="strip-scroller"
      :items="images"
      :item-size="stripItemWidth"
      key-field="baseName"
      direction="horizontal"
      v-slot="{ item, index }"
    >
      <ImageThumbnail
        :image="item"
        :thumbnail="thumbnails.get(item.baseName) || null"
        :selected="selectedIndices.has(index)"
        size="normal"
        @click="(e: MouseEvent) => onImageClick(index, e)"
        @dblclick="() => onImageDblClick(index)"
      />
    </RecycleScroller>

    <ContextMenu
      :visible="menuState.visible"
      :x="menuState.x"
      :y="menuState.y"
      :items="menuState.items"
      @close="hideMenu"
    />
    <ExifDialog
      ref="exifDialogRef"
      :visible="gpsDialogVisible"
      @confirm="handleExifConfirm"
      @cancel="closeExifDialog"
    />
  </div>
</template>

<style scoped>
.image-browser {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
}

/* 向上收起按钮 */
.collapse-up-btn {
  position: absolute;
  top: 4px;
  right: 8px;
  z-index: 10;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--secondary);
  border: 1px solid var(--border);
  border-radius: calc(var(--radius) - 2px);
  cursor: pointer;
  color: var(--foreground);
  transition: all var(--transition-fast);
}

.collapse-up-btn:hover {
  background: var(--accent);
}

/* 空状态 */
.browser-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-xl);
  text-align: center;
}

.empty-icon {
  margin-bottom: var(--spacing-md);
  color: var(--muted-foreground);
}

.empty-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--foreground);
  margin-bottom: var(--spacing-xs);
}

.empty-desc {
  font-size: var(--font-size-sm);
  color: var(--muted-foreground);
}

.browser-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  height: 100%;
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
}

/* Grid 模式滚动器 */
.scroller {
  flex: 1;
  height: 100%;
}

.scroller :deep(.vue-recycle-scroller__item-wrapper) {
  padding: var(--spacing-sm) 0;
}

.thumbnail-row {
  display: flex;
  gap: var(--spacing-sm);
  padding: 0 var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
  width: 100%;
  justify-content: flex-start;
}

/* Strip 模式：横向滚动 (RecycleScroller horizontal) */
.strip-scroller {
  flex: 1;
  height: 100%;
}

.strip-scroller :deep(.vue-recycle-scroller__item-wrapper) {
  display: flex;
  align-items: flex-start;
  padding: var(--spacing-xs);
}
</style>
