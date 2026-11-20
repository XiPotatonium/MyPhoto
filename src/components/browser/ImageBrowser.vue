<script setup lang="ts">
import { ref, watch, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { FolderOpen, Image, RefreshCw, ChevronUp } from 'lucide-vue-next'
import type { ImageGroup, SortField, SortOrder, ExifBatchResult } from '../../types/image'
import type { ExifInfo } from '../../types/exif'
import type {
  BrowserEntry,
  DirectoryContents,
  FolderItem,
} from '../../types/folder'
import { folderKey } from '../../types/folder'
import ImageThumbnail from './ImageThumbnail.vue'
import FolderThumbnail from './FolderThumbnail.vue'
import ContextMenu from '../common/ContextMenu.vue'
import ExifDialog from '../common/ExifDialog.vue'
import type { ExifWriteRequest } from '../common/ExifDialog.vue'
import { useContextMenu } from '../../composables/useContextMenu'
import { RecycleScroller } from 'vue-virtual-scroller'
import { useAppState } from '../../stores/appState'

const appState = useAppState()

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
///
/// exifReady 为 false（EXIF 加载中）时使用与 EXIF 无关的降级键，保证顺序稳定：
/// - name：baseName（不变）
/// - date：仅 fileCreatedTime
/// - rating：baseName 占位
/// EXIF 到齐后（exifReady=true）使用完整键，由 computed 自动一次性重排。
function sortImages(
  images: ImageGroup[],
  sortField: SortField,
  sortOrder: SortOrder,
  exifReady: boolean,
): ImageGroup[] {
  const sorted = [...images]

  switch (sortField) {
    case 'name':
      sorted.sort((a, b) => {
        return a.baseName.toLowerCase().localeCompare(b.baseName.toLowerCase())
      })
      break
    case 'date':
      sorted.sort((a, b) => {
        // 加载中降级：仅用文件创建时间；到齐后：EXIF datetime 回退 fileCreatedTime
        const timeA = exifReady ? getSortTime(a) : a.fileCreatedTime
        const timeB = exifReady ? getSortTime(b) : b.fileCreatedTime
        if (!timeA && !timeB) return 0
        if (!timeA) return 1
        if (!timeB) return -1
        return timeA.localeCompare(timeB)
      })
      break
    case 'rating':
      if (exifReady) {
        sorted.sort((a, b) => {
          const ratingA = getRating(a)
          const ratingB = getRating(b)
          return ratingA - ratingB
        })
      } else {
        // 加载中降级：按 baseName 占位，避免评分渐进填充触发列表跳动
        sorted.sort((a, b) => {
          return a.baseName.toLowerCase().localeCompare(b.baseName.toLowerCase())
        })
      }
      break
  }

  if (sortOrder === 'desc') {
    sorted.reverse()
  }

  return sorted
}

interface ThumbnailResult {
  session_id: string
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
  // 0.3.0：缩略图右下角 i 按钮，交由 App 决定展开右侧详情面板
  'info-requested': [image: ImageGroup]
  // 双击文件夹 / 点击面包屑：由 App 切换当前目录
  'folder-open': [path: string]
  // 空状态里的「打开目录」按钮
  'open-folder-requested': []
  'delete-requested': []
  'collapse-up': []
}>()

// 详情面板已展开且正在展示这张图时，让它的 i 按钮常显高亮
function isInfoActive(img: ImageGroup): boolean {
  return appState.state.infoPanelVisible
    && appState.state.currentImage?.baseName === img.baseName
}

const rawImages = ref<ImageGroup[]>([])
// 0.3.0：当前目录的直属子文件夹，与图片一起渲染在网格里
const folders = ref<FolderItem[]>([])
// baseName → ImageGroup 索引：EXIF/缩略图事件回填时用 O(1) 查找替代线性 find，
// 避免 N 个事件累计成 O(N²) 的字符串比较。
let imageIndex = new Map<string, ImageGroup>()
function rebuildImageIndex() {
  imageIndex = new Map(rawImages.value.map(img => [img.baseName, img]))
}
const loading = ref(false)
const selectedIndex = ref(-1)
const selectedIndices = ref<Set<number>>(new Set())
const lastAnchorIndex = ref(-1)

// 拖拽框选状态
interface DragRect {
  startX: number
  startY: number
  currentX: number
  currentY: number
}
const isDragging = ref(false)
const dragRect = ref<DragRect | null>(null)
const dragSelectionRect = computed(() => {
  if (!dragRect.value) return null
  const { startX, startY, currentX, currentY } = dragRect.value
  return {
    left: Math.min(startX, currentX),
    top: Math.min(startY, currentY),
    width: Math.abs(currentX - startX),
    height: Math.abs(currentY - startY),
  }
})
const thumbnails = ref<Map<string, string>>(new Map())
const { menuState, showMenu, hideMenu } = useContextMenu()
const containerWidth = ref(0)
const exifDialogVisible = ref(false)
const exifDialogRef = ref<InstanceType<typeof ExifDialog> | null>(null)

// 排序后的图片列表（exifReady 由 !exifLoading 提供：EXIF 到齐后自动一次性重排）
const images = computed(() => {
  return sortImages(rawImages.value, props.sortField, props.sortOrder, !appState.state.exifLoading)
})

// 文件夹没有 EXIF / 评分，一律按名称排序；升降序只影响文件夹内部顺序，
// 文件夹整体恒在图片之前（资源管理器习惯，也符合 0.3.0 需求）。
const sortedFolders = computed(() => {
  const list = [...folders.value].sort((a, b) =>
    a.name.toLowerCase().localeCompare(b.name.toLowerCase()),
  )
  if (props.sortOrder === 'desc') list.reverse()
  return list
})

// 网格 / strip 共用的统一条目序列。所有选中、导航、框选索引均基于它。
const entries = computed<BrowserEntry[]>(() => [
  ...sortedFolders.value.map(folder => ({
    kind: 'folder' as const,
    key: folderKey(folder.path),
    folder,
  })),
  ...images.value.map(image => ({
    kind: 'image' as const,
    key: image.baseName,
    image,
  })),
])

/* ------------------------------------------------------------------
   0.3.2 弹性网格：图标边长随窗口宽度连续变化，任意宽度下满行都紧贴两侧

   记 W = 网格可视宽度，P = 行左右 padding(8)，G = 列间距(8)，
   C = 卡片外框额外宽度(padding 4×2 + border 2×2 = 12)，S = 正方形边长，
   n = 列数。满行恰好铺满时：

       n·(S + C) + (n−1)·G + 2P = W
   ⇒   S(n) = (W − 2P − (n−1)·G) / n − C

   列数取 n = ceil((W − 2P + G) / (S_MAX + C + G))：
   这是「使 S(n) ≤ S_MAX」的最小列数。用 ceil 而不是旧的 floor，
   保证剩余宽度由放大图标吃掉（S 不会超过上限），而不是留成右侧空白。
   若 S(n) < S_MIN，说明图标被挤得太小，就减少一列再算（直到 1 列）。
   最后把 S 夹在 [S_MIN, S_MAX]：单列或极窄窗口时宁可居中留白，
   也不把图标拉过 260px 或压到 160px 以下。左右 padding 恒为 P，天然对称。
   ------------------------------------------------------------------ */
const GRID_GAP = 8
const GRID_PADDING = 8
const GRID_CHROME = 12
const GRID_SIZE_MAX = 260 // 现有边长，作为最大边长
const GRID_SIZE_MIN = 160 // 最小边长：低于它就减列而不是继续缩小

const gridMetrics = computed(() => {
  const W = containerWidth.value
  if (W <= 0) return { columns: 4, thumbSize: GRID_SIZE_MAX }

  const usable = W - GRID_PADDING * 2
  let columns = Math.max(
    1,
    Math.ceil((usable + GRID_GAP) / (GRID_SIZE_MAX + GRID_CHROME + GRID_GAP)),
  )
  let thumbSize = 0
  for (;;) {
    thumbSize = (usable - (columns - 1) * GRID_GAP) / columns - GRID_CHROME
    if (thumbSize >= GRID_SIZE_MIN || columns === 1) break
    columns -= 1
  }
  thumbSize = Math.min(GRID_SIZE_MAX, Math.max(GRID_SIZE_MIN, thumbSize))
  return { columns, thumbSize }
})

// 当前模式下的缩略图边长：grid 弹性、strip 固定 130
const currentThumbnailSize = computed(() => (
  props.mode === 'grid' ? gridMetrics.value.thumbSize : 130
))

// 每行显示的条目数（行分组、键盘导航、框选定位都基于它）
const itemsPerRow = computed(() => gridMetrics.value.columns)

// grid 行用等宽列均分：即使测量有亚像素误差或出现垂直滚动条，
// 列宽也会自动吸收误差，不会在右侧空出一整列。
const rowStyle = computed(() => ({
  gridTemplateColumns: `repeat(${gridMetrics.value.columns}, minmax(0, 1fr))`,
}))

// 把弹性边长下发到子组件，覆盖 variables.css 里固定的 --thumbnail-size-large
const gridStyle = computed(() => (
  props.mode === 'grid'
    ? { '--thumbnail-size-large': `${gridMetrics.value.thumbSize}px` }
    : {}
))

// 展平成模板可直接消费的单元格：联合类型在 Vue 模板里收窄不稳定，
// 这里预先把 folder / image 拆成可空字段，避免类型断言。
interface GridCell {
  key: string
  kind: 'folder' | 'image'
  folder: FolderItem | null
  image: ImageGroup | null
  globalIndex: number
}

function toCell(entry: BrowserEntry, globalIndex: number): GridCell {
  return {
    key: entry.key,
    kind: entry.kind,
    folder: entry.kind === 'folder' ? entry.folder : null,
    image: entry.kind === 'image' ? entry.image : null,
    globalIndex,
  }
}

// 将混合条目分组成行（grid 模式）
const imageRows = computed(() => {
  const rows: Array<{ index: number; cells: GridCell[] }> = []
  for (let i = 0; i < entries.value.length; i += itemsPerRow.value) {
    rows.push({
      index: i,
      cells: entries.value
        .slice(i, i + itemsPerRow.value)
        .map((entry, offset) => toCell(entry, i + offset)),
    })
  }
  return rows
})

// strip 模式同样展示文件夹（单击无反应、双击进入并切回浏览视图）
const stripItems = computed(() => entries.value.map((entry, i) => toCell(entry, i)))

// 计算行高度
// thumbnail + 24(label区) + 4(border*2) + 8(行间距)
const rowHeight = computed(() => {
  return currentThumbnailSize.value + 24 + 4 + 8
})

// Strip 模式下单个缩略图的水平宽度 (thumbnail-size + padding)
// thumbnail-size(130) + padding*2(8)
const stripItemWidth = 130 + 12

// 加载会话计数器：每次 loadDirectory 自增并生成唯一 session，用于丢弃旧响应/旧事件，
// 防止快速切换文件夹时旧数据污染。
let loadSessionCounter = 0
const currentSession = ref('')

async function loadDirectory() {
  if (!props.selectedFolder) {
    currentSession.value = ''
    rawImages.value = []
    folders.value = []
    imageIndex.clear()
    appState.setExifLoading(false)
    return
  }
  const session = `s${++loadSessionCounter}`
  currentSession.value = session
  loading.value = true
  try {
    // 带上 sessionId：后端注册会话，让上一轮遗留的缩略图 / EXIF 批量任务立即失效
    const result = await invoke<DirectoryContents>('list_directory_contents', {
      dirPath: props.selectedFolder,
      sessionId: session,
    })
    // 旧响应保护：期间用户已切换文件夹则丢弃结果
    if (currentSession.value !== session) return
    folders.value = result.folders
    rawImages.value = result.images
    rebuildImageIndex()
    selectedIndex.value = -1
    selectedIndices.value.clear()
    thumbnails.value.clear()
    // 列表到达：并发启动缩略图与 EXIF 两个批量任务。
    // 取「当前屏幕顺序」的快照投递：后端严格按传入顺序取任务，
    // 用未排序的原始列表会让首屏图片排到队列末尾才解码。
    appState.setExifLoading(result.images.length > 0)
    const ordered = entries.value.slice()
    loadThumbnailsBatch(session, ordered)
    loadExifBatch(session, ordered)
  } catch (e) {
    if (currentSession.value === session) {
      console.error('Failed to list directory:', e)
      rawImages.value = []
      folders.value = []
      imageIndex.clear()
      appState.setExifLoading(false)
    }
  } finally {
    if (currentSession.value === session) {
      loading.value = false
    }
  }
}

async function loadThumbnailsBatch(session: string, ordered: BrowserEntry[]) {
  // 图片用 baseName、文件夹封面用 folder:<path> 作为回传 key：
  // 封面文件可能与当前目录图片同名，靠唯一 key 避免事件串台。
  const payload = ordered
    .map(entry => {
      const filePath = entry.kind === 'image'
        ? (entry.image.jpgPath || entry.image.rawPath)
        : entry.folder.coverPath
      return filePath ? { key: entry.key, file_path: filePath } : null
    })
    .filter((x): x is { key: string; file_path: string } => x !== null)
  if (payload.length === 0) return

  const unlisten = await listen<ThumbnailResult>('thumbnail-ready', (event) => {
    const { session_id, base_name, thumbnail, error } = event.payload
    // 仅处理当前会话的事件，不匹配一律丢弃
    if (session_id !== session || currentSession.value !== session) return
    if (thumbnail) {
      thumbnails.value.set(base_name, thumbnail)
    } else if (error) {
      console.error(`Failed to generate thumbnail for ${base_name}:`, error)
    }
  })

  try {
    await invoke('generate_thumbnails_batch', {
      filePaths: payload.map(p => p.file_path),
      keys: payload.map(p => p.key),
      sessionId: session,
    })
  } catch (e) {
    console.error('Failed to generate thumbnails batch:', e)
  } finally {
    unlisten()
  }
}

// 两段式加载第二阶段：异步补齐 EXIF。
// 事件渐进填充 + invoke resolve 后对仍缺失的项做补漏回填（不覆盖已有值，
// 保护加载期间用户写入的评分）。
async function loadExifBatch(session: string, ordered: BrowserEntry[]) {
  const items: { base_name: string; file_path: string }[] = []
  for (const entry of ordered) {
    if (entry.kind !== 'image') continue
    const filePath = entry.image.jpgPath || entry.image.rawPath
    if (filePath) items.push({ base_name: entry.image.baseName, file_path: filePath })
  }

  if (items.length === 0) {
    if (currentSession.value === session) {
      appState.setExifLoading(false)
    }
    return
  }

  const unlisten = await listen<ExifBatchResult>('exif-ready', (event) => {
    const payload = event.payload
    if (payload.session_id !== session || currentSession.value !== session) return
    if (!payload.exif_info) return
    const img = imageIndex.get(payload.base_name)
    // 仅填充仍为 null 的项，不覆盖加载期间用户写入的评分
    if (img && !img.exifInfo) {
      img.exifInfo = payload.exif_info
    }
  })

  try {
    const results = await invoke<ExifBatchResult[]>('read_exif_batch', {
      items,
      sessionId: session,
    })
    if (currentSession.value === session) {
      // 补漏回填：仅填充 exifInfo 仍为 null 的项
      for (const r of results) {
        if (!r.exif_info) continue
        const img = imageIndex.get(r.base_name)
        if (img && !img.exifInfo) {
          img.exifInfo = r.exif_info
        }
      }
    }
  } catch (e) {
    console.error('Failed to read exif batch:', e)
  } finally {
    unlisten()
    if (currentSession.value === session) {
      appState.setExifLoading(false)
    }
  }
}

function onImageClick(index: number, e: MouseEvent) {
  // 0.3.0：文件夹单击不做任何交互（不选中、不改当前图）
  const entry = entries.value[index]
  if (!entry || entry.kind !== 'image') return
  const isCtrl = e.ctrlKey || e.metaKey

  if (e.shiftKey && selectedIndex.value >= 0) {
    // Shift 连续选择：从锚点到当前，保留已有选择（若同时按 Ctrl）
    const anchor = lastAnchorIndex.value >= 0 ? lastAnchorIndex.value : selectedIndex.value
    const start = Math.min(anchor, index)
    const end = Math.max(anchor, index)
    if (!isCtrl) {
      selectedIndices.value.clear()
    }
    for (let i = start; i <= end; i++) {
      selectedIndices.value.add(i)
    }
  } else if (isCtrl) {
    // Ctrl/Cmd 点选：切换单个选中状态
    if (selectedIndices.value.has(index)) {
      selectedIndices.value.delete(index)
    } else {
      selectedIndices.value.add(index)
    }
    lastAnchorIndex.value = index
    selectedIndex.value = index
  } else {
    // 普通点击：单选
    selectedIndex.value = index
    lastAnchorIndex.value = index
    selectedIndices.value.clear()
    selectedIndices.value.add(index)
  }
  emit('image-selected', entry.image)
}

function onImageDblClick(index: number) {
  const entry = entries.value[index]
  if (!entry) return
  // 双击文件夹：进入该目录（大图模式下由 App 一并切回浏览视图）
  if (entry.kind === 'folder') {
    emit('folder-open', entry.folder.path)
    return
  }
  selectedIndex.value = index
  selectedIndices.value.clear()
  selectedIndices.value.add(index)
  emit('image-dblclick', entry.image)
}

function onContextMenu(e: MouseEvent) {
  const menuItems = [
    { label: '刷新目录', action: loadDirectory },
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
  exifDialogVisible.value = true
}

function closeExifDialog() {
  exifDialogVisible.value = false
  if (exifDialogRef.value) {
    exifDialogRef.value.reset()
  }
}

async function handleExifConfirm(fields: ExifWriteRequest) {
  if (!exifDialogRef.value) return

  exifDialogRef.value.loading = true

  try {
    const selectedImages: ImageGroup[] = []
    for (const index of selectedIndices.value) {
      const entry = entries.value[index]
      if (entry?.kind === 'image') selectedImages.push(entry.image)
    }
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
    try {
      await invoke('write_exif_fields', { filePaths, fields })
    } catch (e) {
      errors.push(String(e))
    }

    if (errors.length > 0) {
      console.error('Some files failed:', errors)
      alert('部分文件保存失败:\n' + errors.join('\n'))
    } else {
      console.log(`Successfully updated EXIF for ${filePaths.length} file(s)`)
    }

    // 写入成功后重新读取 EXIF 更新 image.exifInfo
    for (const img of selectedImages) {
      const path = img.jpgPath || img.rawPath
      if (path) {
        try {
          const updatedExif = await invoke<ExifInfo>('read_exif', { filePath: path })
          img.exifInfo = updatedExif
        } catch (e) {
          console.error('Failed to re-read EXIF for', img.baseName, e)
        }
      }
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
  if (entries.value.length === 0) return
  // 文件夹恒排在前面，方向键导航要跳过它们
  let newIndex = selectedIndex.value + direction
  while (
    newIndex >= 0 &&
    newIndex < entries.value.length &&
    entries.value[newIndex].kind !== 'image'
  ) {
    newIndex += direction
  }
  if (newIndex < 0 || newIndex >= entries.value.length) return
  const entry = entries.value[newIndex]
  selectedIndex.value = newIndex
  selectedIndices.value.clear()
  selectedIndices.value.add(newIndex)
  if (entry.kind === 'image') emit('image-selected', entry.image)
  ensureIndexVisible(newIndex)
}

// 确保指定索引的缩略图在可视区域内（仅在滚出可视区时做最小滚动）
function ensureIndexVisible(index: number) {
  if (props.mode === 'strip') {
    const el = stripScrollerRef.value?.$el as HTMLElement | undefined
    if (!el) return
    const itemLeft = index * stripItemWidth
    const itemRight = itemLeft + stripItemWidth
    if (itemLeft < el.scrollLeft) {
      el.scrollLeft = itemLeft
    } else if (itemRight > el.scrollLeft + el.clientWidth) {
      el.scrollLeft = itemRight - el.clientWidth
    }
  } else {
    const el = gridScrollerRef.value?.$el as HTMLElement | undefined
    if (!el) return
    const rowIndex = Math.floor(index / itemsPerRow.value)
    const itemTop = rowIndex * rowHeight.value
    const itemBottom = itemTop + rowHeight.value
    if (itemTop < el.scrollTop) {
      el.scrollTop = itemTop
    } else if (itemBottom > el.scrollTop + el.clientHeight) {
      el.scrollTop = itemBottom - el.clientHeight
    }
  }
}

watch(() => props.selectedFolder, loadDirectory)

// 模式切换后恢复滚动位置到选中的图片
const gridScrollerRef = ref<{ scrollToItem: (index: number) => void; $el: HTMLElement } | null>(null)
const stripScrollerRef = ref<{ scrollToItem: (index: number) => void; $el: HTMLElement } | null>(null)

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

// 监听容器宽度变化。
// 网格行宽实际等于滚动器的 clientWidth（已扣掉垂直滚动条），
// 用外层 offsetWidth 会把滚动条那 ~15px 算进可用宽度，导致最后一张卡片溢出。
const browserEl = ref<HTMLElement | null>(null)
let resizeObserver: ResizeObserver | null = null

function measureGridWidth() {
  const scrollerEl = gridScrollerRef.value?.$el as HTMLElement | undefined
  const width = scrollerEl?.clientWidth ?? browserEl.value?.clientWidth ?? 0
  if (width > 0 && width !== containerWidth.value) {
    containerWidth.value = width
  }
}

// 目录内容 / 模式切换会决定滚动条是否出现，DOM 稳定后重新量一次
watch([() => props.mode, () => entries.value.length], () => {
  nextTick(measureGridWidth)
})

// RecycleScroller 只 watch items / sizes，不会监听 item-size 变化。
// 列数不变但窗口变宽时行高会变，必须手动刷新虚拟滚动的行位置，
// 否则 transform 仍按旧行高计算，行与行之间会出现空隙或重叠。
watch(rowHeight, () => {
  nextTick(() => {
    const scroller = gridScrollerRef.value as unknown as {
      updateVisibleItems?: (checkItem: boolean) => void
    } | null
    scroller?.updateVisibleItems?.(true)
  })
})

// 拖拽框选：鼠标按下
function onMouseDown(e: MouseEvent) {
  // 仅处理 grid 模式下的左键拖拽；排除点击在缩略图上的情况
  if (props.mode !== 'grid') return
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  // 如果点击的是缩略图本身，交由 onImageClick 处理；
  // 文件夹卡片单击无交互，也不能作为框选起点
  if (target.closest('.image-thumbnail') || target.closest('.folder-thumbnail')) return

  const container = browserEl.value
  if (!container) return
  const rect = container.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top + (gridScrollerRef.value?.$el?.scrollTop ?? 0)

  isDragging.value = false
  dragRect.value = { startX: x, startY: y, currentX: x, currentY: y }

  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
}

// 拖拽框选：鼠标移动
function onMouseMove(e: MouseEvent) {
  if (!dragRect.value) return
  const container = browserEl.value
  if (!container) return
  const rect = container.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top + (gridScrollerRef.value?.$el?.scrollTop ?? 0)
  dragRect.value.currentX = x
  dragRect.value.currentY = y

  const dx = Math.abs(dragRect.value.currentX - dragRect.value.startX)
  const dy = Math.abs(dragRect.value.currentY - dragRect.value.startY)
  if (dx > 4 || dy > 4) {
    isDragging.value = true
  }

  if (isDragging.value) {
    updateDragSelection(e)
  }
}

// 拖拽框选：计算与框重叠的图片
function updateDragSelection(e: MouseEvent) {
  if (!dragRect.value || !browserEl.value) return

  const container = browserEl.value
  const containerRect = container.getBoundingClientRect()
  const scrollTop = gridScrollerRef.value?.$el?.scrollTop ?? 0

  // 框选区域（相对于容器，考虑滚动）
  const selLeft = Math.min(dragRect.value.startX, dragRect.value.currentX)
  const selTop = Math.min(dragRect.value.startY, dragRect.value.currentY)
  const selRight = Math.max(dragRect.value.startX, dragRect.value.currentX)
  const selBottom = Math.max(dragRect.value.startY, dragRect.value.currentY)

  const newSelected = new Set<number>()
  const thumbnailEls = container.querySelectorAll<HTMLElement>('.image-thumbnail')

  thumbnailEls.forEach((el) => {
    const elRect = el.getBoundingClientRect()
    const elLeft = elRect.left - containerRect.left
    const elTop = elRect.top - containerRect.top + scrollTop
    const elRight = elLeft + elRect.width
    const elBottom = elTop + elRect.height

    // 检测重叠
    if (elRight > selLeft && elLeft < selRight && elBottom > selTop && elTop < selBottom) {
      const indexAttr = el.dataset.index
      if (indexAttr !== undefined) {
        newSelected.add(parseInt(indexAttr))
      }
    }
  })

  // Ctrl 键保留已有选择
  if (e.ctrlKey || e.metaKey) {
    const combined = new Set(selectedIndices.value)
    newSelected.forEach(i => combined.add(i))
    selectedIndices.value = combined
  } else {
    selectedIndices.value = newSelected
  }
}

// 拖拽框选：鼠标释放
function onMouseUp(e: MouseEvent) {
  window.removeEventListener('mousemove', onMouseMove)
  window.removeEventListener('mouseup', onMouseUp)

  if (isDragging.value) {
    updateDragSelection(e)
    if (selectedIndices.value.size > 0) {
      const firstIndex = Math.min(...selectedIndices.value)
      selectedIndex.value = firstIndex
      const entry = entries.value[firstIndex]
      if (entry?.kind === 'image') emit('image-selected', entry.image)
    }
  }

  isDragging.value = false
  dragRect.value = null
}

// Ctrl+A / Cmd+A 全选
function onKeyDown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
    // 全选只选图片，文件夹不参与 EXIF / 删除等操作
    const imageIndices = entries.value.flatMap((entry, i) =>
      entry.kind === 'image' ? [i] : [],
    )
    if (imageIndices.length === 0) return
    e.preventDefault()
    selectedIndices.value = new Set(imageIndices)
    selectedIndex.value = imageIndices[0]
    lastAnchorIndex.value = imageIndices[0]
    const first = entries.value[imageIndices[0]]
    if (first.kind === 'image') emit('image-selected', first.image)
  }
}

onMounted(() => {
  if (browserEl.value) {
    measureGridWidth()
    resizeObserver = new ResizeObserver(() => measureGridWidth())
    resizeObserver.observe(browserEl.value)
  }
  window.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
  window.removeEventListener('mousemove', onMouseMove)
  window.removeEventListener('mouseup', onMouseUp)
  window.removeEventListener('keydown', onKeyDown)
})

// 更新当前选中图片的评分
function updateImageRating(rating: number) {
  if (selectedIndex.value < 0 || selectedIndex.value >= entries.value.length) return

  const entry = entries.value[selectedIndex.value]
  if (!entry || entry.kind !== 'image') return
  const image = entry.image

  const target = imageIndex.get(image.baseName)
  if (!target) return
  if (target.exifInfo) {
    target.exifInfo.rating = rating
  } else {
    // EXIF 尚未加载完成时，创建仅含 rating 的 ExifInfo，避免评分静默丢失
    target.exifInfo = {
      datetime: null,
      gpsLatitude: null,
      gpsLongitude: null,
      cameraMake: null,
      cameraModel: null,
      lensModel: null,
      focalLength: null,
      shutterSpeed: null,
      aperture: null,
      iso: null,
      rating,
      imageWidth: null,
      imageHeight: null,
      orientation: null,
      fileCreatedTime: null,
    }
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
  rebuildImageIndex()

  selectedIndices.value.clear()
  selectedIndex.value = -1

  paths.forEach(path => {
    const baseName = path.split(/[/\\]/).pop()?.split('.').shift()
    if (baseName) {
      thumbnails.value.delete(baseName)
    }
  })
}

// Strip 模式：将纵向滚轮转换为横向滚动
function onStripWheel(e: WheelEvent) {
  const el = stripScrollerRef.value?.$el as HTMLElement | undefined
  if (!el) return
  el.scrollLeft += e.deltaY || e.deltaX
}

defineExpose({
  navigateImage,
  requestDelete,
  selectedIndices,
  images,
  updateImageRating,
  removeImages,
  // 重复打开同一目录时 selectedFolder 不变、watch 不触发，供 App 手动刷新
  reload: loadDirectory,
})
</script>

<template>
  <div
    ref="browserEl"
    class="image-browser"
    :style="gridStyle"
    @contextmenu.prevent="onContextMenu"
    @mousedown="onMouseDown"
  >
    <!-- 0.3.3：路径栏已提升到 App.vue 的 TopToolbar，与打开目录 / 排序 / 主题同一行 -->
    <!-- Strip 模式：向上收起按钮 -->
    <button
      v-if="mode === 'strip'"
      class="collapse-up-btn"
      title="返回浏览视图"
      @click="emit('collapse-up')"
    >
      <ChevronUp class="h-4 w-4" />
    </button>

    <!-- 空状态：未打开目录 -->
    <div v-if="!selectedFolder" class="browser-empty-state">
      <div class="empty-icon">
        <FolderOpen class="h-12 w-12 text-muted-foreground/50" />
      </div>
      <p class="empty-title">未打开目录</p>
      <p class="empty-desc">点击左上角的文件夹按钮，选择一个图库文件夹开始浏览</p>
      <button type="button" class="open-folder-btn" @click="emit('open-folder-requested')">
        <FolderOpen class="h-4 w-4" />
        打开目录
      </button>
    </div>
    <div v-else-if="loading" class="browser-loading">
      <RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
      <span>加载目录中...</span>
    </div>
    <div v-else-if="entries.length === 0" class="browser-empty-state">
      <div class="empty-icon">
        <Image class="h-12 w-12 text-muted-foreground/50" />
      </div>
      <p class="empty-title">空目录</p>
      <p class="empty-desc">此目录中没有子文件夹或支持的图像文件</p>
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
      <div class="thumbnail-row" :style="rowStyle">
        <template v-for="cell in item.cells" :key="cell.key">
          <FolderThumbnail
            v-if="cell.kind === 'folder' && cell.folder"
            :folder="cell.folder"
            :thumbnail="thumbnails.get(cell.key) || null"
            size="large"
            @dblclick="() => onImageDblClick(cell.globalIndex)"
          />
          <ImageThumbnail
            v-else-if="cell.image"
            :image="cell.image"
            :thumbnail="thumbnails.get(cell.key) || null"
            :selected="selectedIndices.has(cell.globalIndex)"
            :data-index="cell.globalIndex"
            size="large"
            :info-active="isInfoActive(cell.image)"
            @click="(e: MouseEvent) => onImageClick(cell.globalIndex, e)"
            @dblclick="() => onImageDblClick(cell.globalIndex)"
            @info-click="(img: ImageGroup) => emit('info-requested', img)"
          />
        </template>
      </div>
    </RecycleScroller>

    <!-- Strip 模式：横向滚动单行 -->
    <RecycleScroller
      v-else
      ref="stripScrollerRef"
      class="strip-scroller"
      :items="stripItems"
      :item-size="stripItemWidth"
      key-field="key"
      direction="horizontal"
      v-slot="{ item, index }"
      @wheel.prevent="onStripWheel"
    >
      <FolderThumbnail
        v-if="item.kind === 'folder' && item.folder"
        :folder="item.folder"
        :thumbnail="thumbnails.get(item.key) || null"
        size="normal"
        @dblclick="() => onImageDblClick(index)"
      />
      <ImageThumbnail
        v-else-if="item.image"
        :image="item.image"
        :thumbnail="thumbnails.get(item.key) || null"
        :selected="selectedIndices.has(index)"
        size="normal"
        :info-active="isInfoActive(item.image)"
        @click="(e: MouseEvent) => onImageClick(index, e)"
        @dblclick="() => onImageDblClick(index)"
        @info-click="(img: ImageGroup) => emit('info-requested', img)"
      />
    </RecycleScroller>

    <!-- 拖拽选择框 -->
    <div
      v-if="isDragging && dragSelectionRect"
      class="drag-selection-box"
      :style="{
        left: dragSelectionRect.left + 'px',
        top: (dragSelectionRect.top - (gridScrollerRef?.$el?.scrollTop ?? 0)) + 'px',
        width: dragSelectionRect.width + 'px',
        height: dragSelectionRect.height + 'px',
      }"
    />

    <!-- EXIF 加载中轻量提示（到齐后会有一次排序重排，此提示让用户有预期） -->
    <div v-if="appState.state.exifLoading && images.length > 0" class="exif-loading-hint">
      <RefreshCw class="h-3 w-3 animate-spin" />
      <span>EXIF 加载中…</span>
    </div>

    <ContextMenu
      :visible="menuState.visible"
      :x="menuState.x"
      :y="menuState.y"
      :items="menuState.items"
      @close="hideMenu"
    />
    <ExifDialog
      ref="exifDialogRef"
      :visible="exifDialogVisible"
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

/* 空状态里的「打开目录」按钮（目录树移除后的主要入口） */
.open-folder-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-top: var(--spacing-md);
  padding: 8px 14px;
  border: 1px solid var(--border);
  border-radius: calc(var(--radius) - 2px);
  background: var(--secondary);
  color: var(--foreground);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.open-folder-btn:hover {
  background: var(--accent);
}

/* Grid 模式滚动器：用 flex 分配剩余高度而不是写死 100% */
.scroller {
  flex: 1 1 0;
  min-height: 0;
}

.scroller :deep(.vue-recycle-scroller__item-wrapper) {
  padding: var(--spacing-sm) 0;
}

/* 等宽列：列数由 gridMetrics 计算，列宽 = (W - 2P - (n-1)G) / n。
   卡片撑满列，内部正方形取 min(100%, 弹性边长)，所以满行左右都只留 8px。 */
.thumbnail-row {
  display: grid;
  gap: var(--spacing-sm);
  padding: 0 var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
  width: 100%;
}

/* Strip 模式：横向滚动 (RecycleScroller horizontal) */
.strip-scroller {
  flex: 1 1 0;
  min-height: 0;
}

.strip-scroller :deep(.vue-recycle-scroller__item-wrapper) {
  display: flex;
  align-items: flex-start;
  padding: var(--spacing-xs);
}

/* 拖拽框选框 */
.drag-selection-box {
  position: absolute;
  pointer-events: none;
  border: 1.5px solid var(--primary);
  background: oklch(from var(--primary) l c h / 0.12);
  border-radius: 3px;
  z-index: 100;
}

/* EXIF 加载中轻量角标提示 */
.exif-loading-hint {
  position: absolute;
  bottom: 8px;
  right: 8px;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  background: oklch(from var(--secondary) l c h / 0.9);
  border: 1px solid var(--border);
  border-radius: calc(var(--radius) - 2px);
  color: var(--muted-foreground);
  font-size: var(--font-size-xs, 11px);
  pointer-events: none;
  user-select: none;
}
</style>
