import { reactive, readonly, watch } from 'vue'
import type { ImageGroup, SortField, SortOrder } from '../types/image'

export type Theme = 'light' | 'dark' | 'system'

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
  theme: Theme
  confirmDialog: ConfirmDialogState
}

const STORAGE_KEY = 'myphoto-theme'

function getInitialTheme(): Theme {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === 'light' || stored === 'dark' || stored === 'system') {
    return stored
  }
  return 'system'
}

function applyTheme(theme: Theme) {
  const root = document.documentElement
  
  if (theme === 'system') {
    root.removeAttribute('data-theme')
  } else {
    root.setAttribute('data-theme', theme)
  }
}

const state = reactive<AppState>({
  rootPath: null,
  selectedFolder: null,
  selectedImages: [],
  currentImage: null,
  currentFormat: 'jpg',
  sortField: 'name',
  sortOrder: 'asc',
  theme: getInitialTheme(),
  confirmDialog: {
    visible: false,
    title: '',
    message: '',
    options: [],
    onConfirm: () => {},
  },
})

// Apply initial theme
applyTheme(state.theme)

// Watch for theme changes
watch(() => state.theme, (newTheme) => {
  localStorage.setItem(STORAGE_KEY, newTheme)
  applyTheme(newTheme)
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

  function setTheme(theme: Theme) {
    state.theme = theme
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
    setTheme,
    showConfirmDialog,
    hideConfirmDialog,
  }
}

export type AppStateReturn = ReturnType<typeof useAppState>
