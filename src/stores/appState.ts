import { reactive, readonly } from 'vue'
import type { ImageGroup, SortField, SortOrder } from '../types/image'

interface ConfirmDialogState {
  visible: boolean
  title: string
  message: string
  options: { label: string; value: string }[]
  onConfirm: (value: string) => void
}

interface AppState {
  rootPath: string | null
  selectedFolder: string | null
  selectedImages: ImageGroup[]
  currentImage: ImageGroup | null
  currentFormat: 'jpg' | 'raw'
  sortField: SortField
  sortOrder: SortOrder
  confirmDialog: ConfirmDialogState
}

const state = reactive<AppState>({
  rootPath: null,
  selectedFolder: null,
  selectedImages: [],
  currentImage: null,
  currentFormat: 'jpg',
  sortField: 'name',
  sortOrder: 'asc',
  confirmDialog: {
    visible: false,
    title: '',
    message: '',
    options: [],
    onConfirm: () => {},
  },
})

export function useAppState() {
  function setRootPath(path: string | null) {
    state.rootPath = path
  }

  function setSelectedFolder(path: string | null) {
    state.selectedFolder = path
  }

  function setSelectedImages(images: ImageGroup[]) {
    state.selectedImages = images
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

  function showConfirmDialog(
    title: string,
    message: string,
    options: { label: string; value: string }[],
    onConfirm: (value: string) => void,
  ) {
    state.confirmDialog = { visible: true, title, message, options, onConfirm }
  }

  function hideConfirmDialog() {
    state.confirmDialog.visible = false
  }

  return {
    state: readonly(state) as AppState,
    setRootPath,
    setSelectedFolder,
    setSelectedImages,
    setCurrentImage,
    setCurrentFormat,
    setSortField,
    setSortOrder,
    showConfirmDialog,
    hideConfirmDialog,
  }
}

export type AppStateReturn = ReturnType<typeof useAppState>
