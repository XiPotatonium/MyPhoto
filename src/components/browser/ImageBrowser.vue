<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { FolderOpen, Image, RefreshCw } from 'lucide-vue-next'
import type { ImageGroup, SortField, SortOrder } from '../../types/image'
import ImageThumbnail from './ImageThumbnail.vue'
import ContextMenu from '../common/ContextMenu.vue'
import GPSDialog from '../common/GPSDialog.vue'
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

const props = defineProps<{
  selectedFolder: string | null
  sortField: SortField
  sortOrder: SortOrder
}>()

const emit = defineEmits<{
  'image-selected': [image: ImageGroup]
  'delete-requested': []
}>()

const rawImages = ref<ImageGroup[]>([])
const loading = ref(false)
const selectedIndex = ref(-1)
const selectedIndices = ref<Set<number>>(new Set())
const thumbnails = ref<Map<string, string>>(new Map())
const { menuState, showMenu, hideMenu } = useContextMenu()
const containerWidth = ref(0)
const gpsDialogVisible = ref(false)
const gpsDialogRef = ref<InstanceType<typeof GPSDialog> | null>(null)

// 排序后的图片列表
const images = computed(() => {
  return sortImages(rawImages.value, props.sortField, props.sortOrder)
})

// 计算每行显示的缩略图数量
const itemsPerRow = computed(() => {
  const thumbnailSize = 130 // 与 CSS 变量 --thumbnail-size 保持一致
  const thumbnailPadding = 8 // 与 CSS 中 .image-thumbnail 的 padding 一致 (var(--spacing-xs) * 2 = 4px * 2)
  const itemTotalWidth = thumbnailSize + thumbnailPadding // 单个缩略图项的总宽度
  const gap = 8 // 与 CSS 中 .thumbnail-row 的 gap 一致 (var(--spacing-sm))
  const padding = 8 // 与 CSS 中 .thumbnail-row 的 padding 一致 (var(--spacing-sm))
  const availableWidth = containerWidth.value - padding * 2

  // 调试输出
  console.log('Container width:', containerWidth.value, 'Available width:', availableWidth, 'Item width:', itemTotalWidth)

  if (availableWidth <= 0 || containerWidth.value === 0) {
    console.log('Using default itemsPerRow: 4')
    return 4 // 默认值
  }

  const result = Math.max(1, Math.floor((availableWidth + gap) / (itemTotalWidth + gap)))
  console.log('Items per row:', result)
  return result
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

// 计算行高度 (thumbnail-size 130px + label 约 20px + padding)
const rowHeight = 130 + 24 + 8 // 缩略图高度 + label高度 + 间距

async function loadImages() {
  if (!props.selectedFolder) {
    rawImages.value = []
    return
  }
  loading.value = true
  try {
    // 后端不再排序，只返回原始数据
    rawImages.value = await invoke<ImageGroup[]>('list_images', {
      dirPath: props.selectedFolder,
    })
    selectedIndex.value = -1
    selectedIndices.value.clear()
    thumbnails.value.clear()
    loadThumbnailsBatch() // 不使用 await，让它在后台运行
  } catch (e) {
    console.error('Failed to list images:', e)
    rawImages.value = []
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
  const menuItems = [
    { label: '刷新文件夹', action: loadImages },
    { label: '删除', action: requestDelete },
  ]

  // 只有在有选中图片时才显示添加GPS信息选项
  if (selectedIndices.value.size > 0) {
    menuItems.push({ label: '添加GPS信息', action: openGPSDialog })
  }

  showMenu(e, menuItems)
}

function requestDelete() {
  // 触发删除请求事件，由父组件统一处理删除逻辑
  emit('delete-requested')
}

function openGPSDialog() {
  if (selectedIndices.value.size === 0) return
  gpsDialogVisible.value = true
}

function closeGPSDialog() {
  gpsDialogVisible.value = false
  if (gpsDialogRef.value) {
    gpsDialogRef.value.reset()
  }
}

async function handleGPSConfirm(latitude: number, longitude: number) {
  if (!gpsDialogRef.value) return

  // 设置加载状态
  gpsDialogRef.value.loading = true

  try {
    // 获取选中图片的路径，同时收集JPG和RAW路径
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

    // 调用批量写入GPS信息命令
    await invoke('batch_write_gps', {
      paths: filePaths,
      latitude,
      longitude,
    })

    console.log(`Successfully added GPS info to ${filePaths.length} file(s)`)

    // 关闭对话框
    closeGPSDialog()
  } catch (e) {
    console.error('Failed to write GPS info:', e)
    alert('保存GPS信息失败: ' + e)
  } finally {
    if (gpsDialogRef.value) {
      gpsDialogRef.value.loading = false
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

// 只在文件夹改变时重新加载图片
watch(() => props.selectedFolder, loadImages)
// 排序方式改变时不需要重新加载图片，computed 属性 images 会自动重新排序

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

  // 找到原始图片数据并更新
  const rawIndex = rawImages.value.findIndex(img => img.baseName === image.baseName)
  if (rawIndex >= 0 && rawImages.value[rawIndex].exifInfo) {
    rawImages.value[rawIndex].exifInfo!.rating = rating
  }
}

// 前端直接移除图片
function removeImages(paths: string[]) {
  const pathSet = new Set(paths)

  // 过滤掉被删除的图片
  const newRawImages = rawImages.value.filter(img => {
    const jpgDeleted = img.jpgPath && pathSet.has(img.jpgPath)
    const rawDeleted = img.rawPath && pathSet.has(img.rawPath)

    // 如果只删除了其中一个格式，更新图片组而不是移除
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

    // 如果两个都被删除，或者只有一个格式且被删除，则移除
    return !(jpgDeleted || rawDeleted)
  })

  rawImages.value = newRawImages

  // 清除选中状态
  selectedIndices.value.clear()
  selectedIndex.value = -1

  // 清除缩略图缓存
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
  <div ref="browserEl" class="image-browser" @contextmenu.prevent="onContextMenu">
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
    <RecycleScroller
      v-else
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
    <GPSDialog
      ref="gpsDialogRef"
      :visible="gpsDialogVisible"
      @confirm="handleGPSConfirm"
      @cancel="closeGPSDialog"
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
  color: hsl(var(--muted-foreground));
}

.empty-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: var(--spacing-xs);
}

.empty-desc {
  font-size: var(--font-size-sm);
  color: hsl(var(--muted-foreground));
}

.browser-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  height: 100%;
  color: hsl(var(--muted-foreground));
  font-size: var(--font-size-sm);
}

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

/* 保留原 grid 样式作为 fallback */
.thumbnail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--thumbnail-size), 1fr));
  gap: var(--spacing-sm);
}
</style>
