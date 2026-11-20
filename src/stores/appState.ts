import { reactive } from 'vue'
import type { ImageGroup, SortField, SortOrder } from '../types/image'

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
})

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
  }
}

export type AppStateReturn = ReturnType<typeof useAppState>
