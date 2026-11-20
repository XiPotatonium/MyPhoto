<script setup lang="ts">
import { provide, ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { X } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { useAppState } from './stores/appState'
import { useTheme } from './composables/useTheme'
import type { ImageGroup } from './types/image'
import RightPanel from './components/layout/RightPanel.vue'
import ImageBrowser from './components/browser/ImageBrowser.vue'
import ImageViewer from './components/viewer/ImageViewer.vue'
import FormatTabBar from './components/viewer/FormatTabBar.vue'
import ExifDisplay from './components/info/ExifDisplay.vue'
import ConfirmDialog from './components/common/ConfirmDialog.vue'

async function writeRating(image: ImageGroup, rating: number): Promise<boolean> {
  const filePath = image.jpgPath || image.rawPath
  if (!filePath) return false
  try {
    await invoke('write_rating', { filePath, rating })
    // 写入成功后，重新读取 EXIF 并更新 image.exifInfo
    const updatedExif = await invoke<import('./types/exif').ExifInfo>('read_exif', { filePath })
    image.exifInfo = updatedExif
    return true
  } catch (e) {
    console.error('Failed to write rating:', e)
    return false
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
    const oldRating = image.exifInfo?.rating ?? 0
    imageBrowserRef.value?.updateImageRating(rating)
    writeRating(image, rating).then((success) => {
      if (!success) {
        imageBrowserRef.value?.updateImageRating(oldRating)
      }
    })
  }

  if (e.key === 'Escape') {
    // 详情面板是按需展开的浮层语义，优先收起它；否则维持原有的退出大图行为
    if (appState.state.infoPanelVisible) {
      e.preventDefault()
      appState.setInfoPanelVisible(false)
      return
    }
    if (appState.state.viewMode === 'view') {
      e.preventDefault()
      appState.setViewMode('browse')
    }
  }

  // i：快捷切换右侧详情面板（带修饰键时不拦截，留给系统/浏览器快捷键）
  if (
    e.key.toLowerCase() === 'i' &&
    !e.ctrlKey && !e.metaKey && !e.altKey &&
    appState.state.currentImage
  ) {
    e.preventDefault()
    appState.toggleInfoPanel()
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

// 0.3.0：缩略图 i 按钮。点其他图则切过去并展开；
// 点当前正在展示详情的那张图则收起，与按钮的「收起图片信息」提示保持一致。
function handleInfoRequested(image: ImageGroup) {
  const isSameActive =
    appState.state.infoPanelVisible &&
    appState.state.currentImage?.baseName === image.baseName
  if (isSameActive) {
    appState.setInfoPanelVisible(false)
    return
  }
  appState.setCurrentImage(image)
  appState.setInfoPanelVisible(true)
}

function handleInfoClose() {
  appState.setInfoPanelVisible(false)
}

// 0.3.0：双击文件夹卡片 / 点击路径面包屑 → 进入该目录。
// 大图状态下还会切回文件浏览器模式。
function handleFolderOpen(path: string) {
  appState.setSelectedFolder(path)
  appState.setCurrentImage(null)
  appState.setViewMode('browse')
}

// 0.3.0：目录树移除后，「设置根目录」迁入原生「文件 → 打开目录」菜单，
// 空状态里的「打开目录」按钮也复用这里。
async function openRootDirectory() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: false })
    if (!selected) return
    const path = selected as string
    const sameFolder = appState.state.selectedFolder === path
    appState.setRootPath(path)
    appState.setSelectedFolder(path)
    appState.setCurrentImage(null)
    appState.setViewMode('browse')
    // 重复打开同一目录时 selectedFolder 不变，watch 不会触发，手动刷新一次
    if (sameFolder) imageBrowserRef.value?.reload()
  } catch (e) {
    console.error('Failed to open folder dialog:', e)
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
  const unlistenOpenFolder = await listen('menu-open-folder', () => {
    openRootDirectory()
  })

  unlistenCallbacks = [
    unlistenSortField,
    unlistenSortOrder,
    unlistenTheme,
    unlistenOpenFolder,
  ]
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  unlistenCallbacks.forEach((fn) => fn())
})
</script>

<template>
  <div class="app-container">
    <div
      :class="[
        'main-layout',
        appState.state.viewMode === 'view' && 'main-layout--view',
        appState.state.infoPanelVisible && 'main-layout--info-open',
      ]"
    >
      <!-- 0.3.0：左侧目录树已移除，文件浏览器自身承担目录导航 -->
      <!-- 中列：通过动画切换布局 -->
      <div class="center-panel-container">
        <!-- 图像查看区域：browse 模式下高度 0，view 模式下占满上方 -->
        <div :class="['viewer-area', appState.state.viewMode === 'view' && 'viewer-area--active']">
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
            :root-path="appState.state.rootPath"
            :sort-field="appState.state.sortField"
            :sort-order="appState.state.sortOrder"
            @image-selected="handleImageSelected"
            @image-dblclick="handleImageDblClick"
            @info-requested="handleInfoRequested"
            @folder-open="handleFolderOpen"
            @open-folder-requested="openRootDirectory"
            @delete-requested="handleDelete"
            @collapse-up="handleCollapseUp"
          />
        </div>
      </div>

      <!-- 右列：照片信息。0.3.0 起不再常驻，由缩略图上的 i 按钮按需展开。
           用 flex 宽度伸缩挤压中间区域，而不是 absolute 浮层覆盖，保证不遮挡图片；
           大图状态下底部恒截止到横向浏览器上沿，把下方宽度让给 strip。 -->
      <div
        :class="[
          'right-panel-container',
          appState.state.infoPanelVisible && 'right-panel-container--open'
        ]"
      >
        <!-- 内层固定宽度：容器从 0 展开时内容不会被压扁，形成抽屉拉开效果 -->
        <div v-show="appState.state.infoPanelVisible" class="right-panel-inner">
          <!-- 悬浮卡片：四周留白 + 圆角 + 阴影，不再上下左右顶满 -->
          <div class="info-panel-card">
            <RightPanel>
              <div class="info-panel-header">
                <span class="info-panel-title">照片信息</span>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  class="info-panel-close"
                  title="关闭图片信息"
                  aria-label="关闭图片信息"
                  @click="handleInfoClose"
                >
                  <X class="h-4 w-4" />
                </Button>
              </div>
              <FormatTabBar
                v-if="appState.state.currentImage"
                :image="appState.state.currentImage"
                :current-format="appState.state.currentFormat"
                :disabled="appState.state.viewMode !== 'view'"
                @update:format="appState.setCurrentFormat"
              />
              <ExifDisplay />
            </RightPanel>
          </div>
        </div>
      </div>
    </div>
    <ConfirmDialog
      :visible="appState.state.confirmDialog.visible"
      :title="appState.state.confirmDialog.title"
      :message="appState.state.confirmDialog.message"
      :options="appState.state.confirmDialog.options"
      @confirm="async (v: string) => { try { await appState.state.confirmDialog.onConfirm(v) } finally { appState.hideConfirmDialog() } }"
      @cancel="async () => { try { await appState.state.confirmDialog.onCancel?.() } finally { appState.hideConfirmDialog() } }"
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
  /* 横向浏览器高度：右面板底部与 strip 共用这个界限 */
  --strip-height: 190px;
}

.center-panel-container {
  flex: 1;
  min-width: 200px;
  /* 需要让 strip 用负 margin 向右延伸到右列让出的区域；
     viewer-area / browser-area 各自 overflow:hidden 负责裁剪 */
  overflow: visible;
  display: flex;
  flex-direction: column;
}

/* 0.3.0：右侧详情面板改为按需展开。
   关键：它始终是 main-layout 的 flex 成员，靠宽度变化挤压 center，
   而不是 absolute 浮层覆盖图片。 */
.right-panel-container {
  position: relative;
  flex: 0 0 auto;
  align-self: flex-start;
  width: 0;
  height: 100%;
  min-width: 0;
  overflow: hidden;
  /* 容器只做布局占位：不画底色、不抬层级、不拦截鼠标。
     基础状态不带 transition —— 收起时宽度立即归零，根本不产生会露白的中间帧。 */
  background: transparent;
  pointer-events: none;
  visibility: hidden;
}

.right-panel-container--open {
  width: var(--right-panel-width);
  visibility: visible;
  /* 只有展开时做抽屉拉开动画 */
  transition: width 300ms cubic-bezier(0.4, 0, 0.2, 1);
}

/* 大图状态下容器高度恒截止到横向浏览器上沿，与面板开合无关：
   否则「收起」移除 info-open 类的瞬间 height 会跳回 100%，
   整列跨到 strip 上方，动画中间帧出现白色遮挡。 */
.main-layout--view .right-panel-container {
  height: calc(100% - var(--strip-height));
}

/* 内层保持固定宽度，容器由 0 展开时内容不被压扁，形成抽屉拉开效果 */
.right-panel-inner {
  width: var(--right-panel-width);
  height: 100%;
  padding: 12px;
  box-sizing: border-box;
}

/* 悬浮卡片：四周留白、圆角、柔和阴影（阴影落在 inner 的 padding 内，
   不会被容器的 overflow:hidden 裁掉）。
   高度随内容收缩，避免内容下方拖出一大片空白；max-height 是上限，
   大图状态下容器的高度已截止到 strip 上沿，因此卡片也不会压到 strip。 */
.info-panel-card {
  width: 100%;
  height: auto;
  max-height: 100%;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--secondary);
  overflow: hidden;
  /* 容器已设 pointer-events:none，只有卡片本体恢复交互 */
  pointer-events: auto;
  box-shadow: 0 6px 16px -6px rgb(0 0 0 / 0.22),
              0 2px 6px -2px rgb(0 0 0 / 0.10);
}

/* RightPanel 自身写死 height:100%，这里改为随内容伸缩；内容超过上限时
   由它自己的 overflow-y:auto 滚动，卡片不再被撑长 */
.info-panel-card :deep(.right-panel) {
  height: auto;
  flex: 1 1 auto;
  min-height: 0;
}

:global(html.dark) .info-panel-card {
  box-shadow: 0 8px 20px -6px rgb(0 0 0 / 0.60),
              0 2px 8px -2px rgb(0 0 0 / 0.40);
}

.info-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-sm);
  flex-shrink: 0;
}

.info-panel-title {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--foreground);
}

.info-panel-close {
  margin-left: auto;
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
  position: relative;
  /* strip 始终绘制在右列之上，任何残留的白色都不可能盖住缩略图 */
  z-index: 1;
  flex: 0 0 var(--strip-height);
  background: var(--background);
  border-top: 1px solid var(--border);
}

/* 大图 + 面板展开：右面板已让出下方高度，strip 向右补足右列宽度，
   横向浏览器因此能一直延伸到窗口右边缘，且不遮挡上方大图。
   transition 只写在展开态：收起时规则连同过渡一起消失，margin 立即回 0，
   与右列宽度立即归零严格同步，不会出现 strip 先缩、右列后缩的白色空隙。 */
.main-layout--view.main-layout--info-open .browser-area--strip {
  margin-right: calc(-1 * var(--right-panel-width));
  transition: flex-grow 350ms cubic-bezier(0.4, 0, 0.2, 1),
              flex-basis 350ms cubic-bezier(0.4, 0, 0.2, 1),
              margin-right 300ms cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
