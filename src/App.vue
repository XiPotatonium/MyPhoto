<script setup lang="ts">
import { provide, ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useAppState } from './stores/appState'
import { useTheme } from './composables/useTheme'
import type { ImageGroup } from './types/image'
import LeftPanel from './components/layout/LeftPanel.vue'
import RightPanel from './components/layout/RightPanel.vue'
import DirectoryTree from './components/directory/DirectoryTree.vue'
import ImageBrowser from './components/browser/ImageBrowser.vue'
import ImageViewer from './components/viewer/ImageViewer.vue'
import FormatTabBar from './components/viewer/FormatTabBar.vue'
import ExifDisplay from './components/info/ExifDisplay.vue'
import ConfirmDialog from './components/common/ConfirmDialog.vue'

async function writeRating(image: ImageGroup, rating: number) {
  const filePath = image.jpgPath || image.rawPath
  if (!filePath) return
  try {
    await invoke('write_rating', { filePath, rating })
  } catch (e) {
    console.error('Failed to write rating:', e)
  }
}

const appState = useAppState()
provide('appState', appState)

// Initialize theme
const { setTheme } = useTheme()

const imageBrowserRef = ref<InstanceType<typeof ImageBrowser> | null>(null)

const browserMode = computed(() => {
  return appState.state.viewMode === 'view' ? 'strip' : 'grid'
})

function onKeyDown(e: KeyboardEvent) {
  const target = e.target as HTMLElement
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT') {
    return
  }

  if (e.key === 'ArrowLeft') {
    e.preventDefault()
    imageBrowserRef.value?.navigateImage(-1)
  } else if (e.key === 'ArrowRight') {
    e.preventDefault()
    imageBrowserRef.value?.navigateImage(1)
  }

  if (e.key === 'Delete' && appState.state.currentImage) {
    e.preventDefault()
    handleDelete()
  }

  if (e.key >= '0' && e.key <= '5' && appState.state.currentImage) {
    e.preventDefault()
    const rating = parseInt(e.key)
    const image = appState.state.currentImage
    writeRating(image, rating)
    imageBrowserRef.value?.updateImageRating(rating)
  }

  if (e.key === 'Escape' && appState.state.viewMode === 'view') {
    e.preventDefault()
    appState.setViewMode('browse')
  }
}

function handleImageSelected(image: ImageGroup) {
  appState.setCurrentImage(image)
}

function handleImageDblClick(image: ImageGroup) {
  appState.setCurrentImage(image)
  appState.setViewMode('view')
}

function handleCollapseUp() {
  appState.setViewMode('browse')
}

function handleFolderSelected(path: string) {
  appState.setSelectedFolder(path)
  appState.setCurrentImage(null)
  if (appState.state.viewMode === 'view') {
    appState.setViewMode('browse')
  }
}

function handleDelete() {
  const image = appState.state.currentImage
  if (!image) return

  const hasJpg = !!image.jpgPath
  const hasRaw = !!image.rawPath
  const hasBoth = hasJpg && hasRaw

  const options: { label: string; value: string }[] = []
  if (hasBoth) {
    options.push({ label: '仅删除 JPG', value: 'jpg' })
    options.push({ label: '仅删除 RAW', value: 'raw' })
    options.push({ label: '全部删除', value: 'all' })
  } else {
    options.push({ label: '确认删除', value: 'all' })
  }

  appState.showConfirmDialog(
    '删除确认',
    `确定要删除 "${image.baseName}" 吗？文件将被移动到回收站。`,
    options,
    async (value: string) => {
      const paths: string[] = []
      if ((value === 'jpg' || value === 'all') && image.jpgPath) {
        paths.push(image.jpgPath)
      }
      if ((value === 'raw' || value === 'all') && image.rawPath) {
        paths.push(image.rawPath)
      }
      if (paths.length > 0) {
        try {
          await invoke('move_to_trash', { paths })
          imageBrowserRef.value?.removeImages(paths)
        } catch (err) {
          console.error('Failed to delete:', err)
        }
      }
    },
  )
}

let unlistenCallbacks: Array<() => void> = []

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)

  // Listen for native menu events
  const unlistenSortField = await listen<string>('menu-sort-field', (event) => {
    appState.setSortField(event.payload as 'name' | 'date' | 'rating')
  })
  const unlistenSortOrder = await listen<string>('menu-sort-order', (event) => {
    appState.setSortOrder(event.payload as 'asc' | 'desc')
  })
  const unlistenTheme = await listen<string>('menu-theme', (event) => {
    setTheme(event.payload as 'light' | 'dark')
  })

  unlistenCallbacks = [unlistenSortField, unlistenSortOrder, unlistenTheme]
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  unlistenCallbacks.forEach((fn) => fn())
})
</script>

<template>
  <div class="app-container">
    <div class="main-layout">
      <!-- 左列：目录树，占满整列 -->
      <div class="left-panel-container">
        <LeftPanel>
          <DirectoryTree
            :root-path="appState.state.rootPath"
            @folder-selected="handleFolderSelected"
            @root-changed="appState.setRootPath"
          />
        </LeftPanel>
      </div>

      <!-- 中列：通过动画切换布局 -->
      <div class="center-panel-container">
        <!-- 图像查看区域：browse 模式下高度 0，view 模式下占满上方 -->
        <div :class="['viewer-area', appState.state.viewMode === 'view' && 'viewer-area--active']">
          <FormatTabBar
            v-if="appState.state.currentImage"
            :image="appState.state.currentImage"
            :current-format="appState.state.currentFormat"
            @update:format="appState.setCurrentFormat"
          />
          <ImageViewer
            :image="appState.state.currentImage"
            :format="appState.state.currentFormat"
          />
        </div>

        <!-- 浏览器区域：browse 模式下占满，view 模式下变为横向条 -->
        <div :class="['browser-area', appState.state.viewMode === 'view' && 'browser-area--strip']">
          <ImageBrowser
            ref="imageBrowserRef"
            :mode="browserMode"
            :selected-folder="appState.state.selectedFolder"
            :sort-field="appState.state.sortField"
            :sort-order="appState.state.sortOrder"
            @image-selected="handleImageSelected"
            @image-dblclick="handleImageDblClick"
            @delete-requested="handleDelete"
            @collapse-up="handleCollapseUp"
          />
        </div>
      </div>

      <!-- 右列：照片信息 -->
      <div class="right-panel-container">
        <RightPanel>
          <ExifDisplay
            :image="appState.state.currentImage"
          />
        </RightPanel>
      </div>
    </div>
    <ConfirmDialog
      :visible="appState.state.confirmDialog.visible"
      :title="appState.state.confirmDialog.title"
      :message="appState.state.confirmDialog.message"
      :options="appState.state.confirmDialog.options"
      @confirm="(v: string) => { appState.state.confirmDialog.onConfirm(v); appState.hideConfirmDialog() }"
      @cancel="appState.hideConfirmDialog"
    />
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.main-layout {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.left-panel-container {
  width: var(--left-panel-width);
  min-width: 200px;
  max-width: 500px;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
  overflow: hidden;
}

.center-panel-container {
  flex: 1;
  min-width: 200px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.right-panel-container {
  width: var(--right-panel-width);
  min-width: 200px;
  max-width: 400px;
  flex-shrink: 0;
  border-left: 1px solid var(--border);
  overflow: hidden;
}

/* ========================================
   视图区域 - 动画切换
   使用纯数值 flex-grow/flex-basis 确保可动画
   ======================================== */

/* 图像查看区域 */
.viewer-area {
  flex: 0 0 0px;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--background);
  opacity: 0;
  transition: flex-grow 350ms cubic-bezier(0.4, 0, 0.2, 1),
              opacity 250ms ease;
}

.viewer-area--active {
  flex: 1 1 0px;
  opacity: 1;
  border-bottom: 1px solid var(--border);
}

/* 浏览器区域 */
.browser-area {
  flex: 1 1 0px;
  min-height: 0;
  overflow: hidden;
  background: var(--background);
  transition: flex-grow 350ms cubic-bezier(0.4, 0, 0.2, 1),
              flex-basis 350ms cubic-bezier(0.4, 0, 0.2, 1);
}

.browser-area--strip {
  flex: 0 0 190px;
  background: var(--background);
  border-top: 1px solid var(--border);
}
</style>
