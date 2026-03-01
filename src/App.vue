<script setup lang="ts">
import { provide, ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppState } from './stores/appState'
import { useTheme } from './composables/useTheme'
import type { ImageGroup } from './types/image'
import TopFilterBar from './components/layout/TopFilterBar.vue'
import LeftPanel from './components/layout/LeftPanel.vue'
import CenterPanel from './components/layout/CenterPanel.vue'
import RightPanel from './components/layout/RightPanel.vue'
import DirectoryTree from './components/directory/DirectoryTree.vue'
import ImageBrowser from './components/browser/ImageBrowser.vue'
import ImageViewer from './components/viewer/ImageViewer.vue'
import FormatTabBar from './components/viewer/FormatTabBar.vue'
import ExifDisplay from './components/info/ExifDisplay.vue'
import ConfirmDialog from './components/common/ConfirmDialog.vue'

const appState = useAppState()
provide('appState', appState)

// Initialize theme
useTheme()

const imageBrowserRef = ref<InstanceType<typeof ImageBrowser> | null>(null)
const exifDisplayRef = ref<InstanceType<typeof ExifDisplay> | null>(null)

function onKeyDown(e: KeyboardEvent) {
  // Ignore if user is typing in an input
  const target = e.target as HTMLElement
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT') {
    return
  }

  // Arrow keys: navigate images
  if (e.key === 'ArrowLeft') {
    e.preventDefault()
    imageBrowserRef.value?.navigateImage(-1)
  } else if (e.key === 'ArrowRight') {
    e.preventDefault()
    imageBrowserRef.value?.navigateImage(1)
  }

  // Delete key: delete selected images
  if (e.key === 'Delete' && appState.state.currentImage) {
    e.preventDefault()
    handleDelete()
  }

  // 0-5: set rating
  if (e.key >= '0' && e.key <= '5' && appState.state.currentImage) {
    e.preventDefault()
    exifDisplayRef.value?.setRating(parseInt(e.key))
  }
}

function handleRatingUpdated(_image: ImageGroup, rating: number) {
  // 同步更新 ImageBrowser 中的图片信息
  imageBrowserRef.value?.updateImageRating(rating)
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
          // 前端直接移除图片，而不是重新调用 list_images
          imageBrowserRef.value?.removeImages(paths)
        } catch (err) {
          console.error('Failed to delete:', err)
        }
      }
    },
  )
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
})
</script>

<template>
  <div class="app-container">
    <TopFilterBar
      :sort-field="appState.state.sortField"
      :sort-order="appState.state.sortOrder"
      @update:sort-field="appState.setSortField"
      @update:sort-order="appState.setSortOrder"
    />
    <div class="main-layout">
      <div class="left-panel-container">
        <LeftPanel>
          <template #tree>
            <DirectoryTree
              :root-path="appState.state.rootPath"
              @folder-selected="appState.setSelectedFolder"
              @root-changed="appState.setRootPath"
            />
          </template>
          <template #browser>
            <ImageBrowser
              ref="imageBrowserRef"
              :selected-folder="appState.state.selectedFolder"
              :sort-field="appState.state.sortField"
              :sort-order="appState.state.sortOrder"
              @image-selected="appState.setCurrentImage"
              @delete-requested="handleDelete"
            />
          </template>
        </LeftPanel>
      </div>
      <div class="center-panel-container">
        <CenterPanel>
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
        </CenterPanel>
      </div>
      <div class="right-panel-container">
        <RightPanel>
          <ExifDisplay 
            ref="exifDisplayRef"
            :image="appState.state.currentImage" 
            @rating-updated="handleRatingUpdated"
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
  border-right: 1px solid hsl(var(--border));
  overflow: hidden;
}

.center-panel-container {
  flex: 1;
  min-width: 200px;
  overflow: hidden;
}

.right-panel-container {
  width: var(--right-panel-width);
  min-width: 200px;
  max-width: 400px;
  flex-shrink: 0;
  border-left: 1px solid hsl(var(--border));
  overflow: hidden;
}
</style>
