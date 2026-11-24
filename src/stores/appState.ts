import { reactive } from 'vue'
import type { ImageGroup, SortField, SortOrder } from '../types/image'
import type { TaskKind, TaskProgress } from '../types/task'

interface ConfirmDialogState {
  visible: boolean
  title: string
  message: string
  options: { label: string; value: string }[]
  onConfirm: (value: string) => void
  onCancel?: () => void | Promise<void>
}

export type ViewMode = 'browse' | 'view'

interface AppState {
  rootPath: string | null
  selectedFolder: string | null
  currentImage: ImageGroup | null
  currentFormat: 'jpg' | 'raw'
  sortField: SortField
  sortOrder: SortOrder
  viewMode: ViewMode
  exifLoading: boolean
  // 0.3.0：右侧详情页不再常驻，由缩略图 / 大图控件上的 i 按钮按需展开
  infoPanelVisible: boolean
  confirmDialog: ConfirmDialogState
  // 0.3.4：HDR 转换 / EXIF 写入等后台任务，左上角进度浮层据此渲染
  tasks: TaskProgress[]
}

const state = reactive<AppState>({
  rootPath: null,
  selectedFolder: null,
  currentImage: null,
  currentFormat: 'jpg',
  sortField: 'name',
  sortOrder: 'asc',
  viewMode: 'browse',
  exifLoading: false,
  infoPanelVisible: false,
  confirmDialog: {
    visible: false,
    title: '',
    message: '',
    options: [],
    onConfirm: () => {},
    onCancel: undefined,
  },
  tasks: [],
})

let taskCounter = 0

export function useAppState() {
  function setRootPath(path: string | null) {
    state.rootPath = path
  }

  function setSelectedFolder(path: string | null) {
    state.selectedFolder = path
  }

  function setCurrentImage(image: ImageGroup | null) {
    state.currentImage = image
    if (image) {
      state.currentFormat = image.jpgPath ? 'jpg' : 'raw'
    }
  }

  function setCurrentFormat(format: 'jpg' | 'raw') {
    state.currentFormat = format
  }

  function setSortField(field: SortField) {
    state.sortField = field
  }

  function setSortOrder(order: SortOrder) {
    state.sortOrder = order
  }

  function setViewMode(mode: ViewMode) {
    state.viewMode = mode
  }

  function setExifLoading(loading: boolean) {
    state.exifLoading = loading
  }

  function setInfoPanelVisible(visible: boolean) {
    state.infoPanelVisible = visible
  }

  function toggleInfoPanel() {
    state.infoPanelVisible = !state.infoPanelVisible
  }

  function showConfirmDialog(
    title: string,
    message: string,
    options: { label: string; value: string }[],
    onConfirm: (value: string) => void,
    onCancel?: () => void | Promise<void>,
  ) {
    state.confirmDialog = { visible: true, title, message, options, onConfirm, onCancel }
  }

  function hideConfirmDialog() {
    state.confirmDialog.visible = false
  }

  // ── 后台任务进度（0.3.4）─────────────────────────────────────────────────

  function startTask(kind: TaskKind, title: string, total: number): string {
    const id = `${kind}-${Date.now()}-${++taskCounter}`
    state.tasks.push({
      id,
      kind,
      title,
      processed: 0,
      total: Math.max(total, 0),
      current: null,
      failed: 0,
      done: false,
    })
    return id
  }

  function updateTask(
    id: string,
    patch: Partial<Pick<TaskProgress, 'processed' | 'total' | 'current' | 'failed'>>,
  ) {
    const task = state.tasks.find(item => item.id === id)
    if (!task) return
    Object.assign(task, patch)
    if (task.processed > task.total) task.processed = task.total
  }

  function finishTask(id: string, failed?: number) {
    const task = state.tasks.find(item => item.id === id)
    if (!task) return
    if (typeof failed === 'number') task.failed = failed
    task.processed = task.total
    task.current = null
    task.done = true
  }

  function removeTask(id: string) {
    state.tasks = state.tasks.filter(item => item.id !== id)
  }

  return {
    state,
    setRootPath,
    setSelectedFolder,
    setCurrentImage,
    setCurrentFormat,
    setSortField,
    setSortOrder,
    setViewMode,
    setExifLoading,
    setInfoPanelVisible,
    toggleInfoPanel,
    showConfirmDialog,
    hideConfirmDialog,
    startTask,
    updateTask,
    finishTask,
    removeTask,
  }
}

export type AppStateReturn = ReturnType<typeof useAppState>
