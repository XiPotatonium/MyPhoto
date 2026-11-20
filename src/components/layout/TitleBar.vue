<script setup lang="ts">
import { Aperture, Minus, Square, X } from 'lucide-vue-next'
import { getCurrentWindow } from '@tauri-apps/api/window'

// 0.3.3：decorations:false 后窗口没有系统标题栏，这里的三个按钮
// 直接调用 Tauri 窗口 API；权限在 capabilities/default.json 中声明。
// 惰性取窗口对象，避免在非 Tauri 环境（如浏览器预览）里初始化就抛错。
function win() {
  try {
    return getCurrentWindow()
  } catch {
    return null
  }
}

async function minimize() {
  try { await win()?.minimize() } catch (e) { console.error('minimize failed:', e) }
}
async function toggleMaximize() {
  try { await win()?.toggleMaximize() } catch (e) { console.error('toggleMaximize failed:', e) }
}
async function close() {
  try { await win()?.close() } catch (e) { console.error('close failed:', e) }
}
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-brand" data-tauri-drag-region>
      <span class="titlebar-logo" data-tauri-drag-region>
        <Aperture class="h-4 w-4" />
      </span>
      <span class="titlebar-title" data-tauri-drag-region>MyPhoto</span>
    </div>

    <!-- 中间空白同样是拖拽区，双击由 Tauri 自动切换最大化 -->
    <div class="titlebar-spacer" data-tauri-drag-region />

    <div class="titlebar-controls">
      <button
        type="button"
        class="titlebar-btn"
        title="最小化"
        aria-label="最小化"
        @click="minimize"
      >
        <Minus class="h-4 w-4" />
      </button>
      <button
        type="button"
        class="titlebar-btn"
        title="最大化 / 还原"
        aria-label="最大化 / 还原"
        @click="toggleMaximize"
      >
        <Square class="h-3.5 w-3.5" />
      </button>
      <button
        type="button"
        class="titlebar-btn titlebar-btn--close"
        title="关闭"
        aria-label="关闭"
        @click="close"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 背景完全透明：不画自己的底色，直接继承 body 的 --background，
   标题栏与下方工具栏、内容区融为一整块，浅色/深色主题都自动适配。 */
.titlebar {
  flex: 0 0 46px;
  display: flex;
  align-items: center;
  height: 46px;
  padding-left: 14px;
  background: transparent;
  user-select: none;
  -webkit-app-region: drag;
  app-region: drag;
}

.titlebar-brand {
  display: flex;
  align-items: center;
  gap: 9px;
  flex-shrink: 0;
}

.titlebar-logo {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 7px;
  background: var(--primary);
  color: var(--primary-foreground);
}

.titlebar-title {
  font-size: 15.5px;
  font-weight: 600;
  letter-spacing: 0.01em;
  color: var(--foreground);
}

.titlebar-spacer {
  flex: 1 1 auto;
  min-width: 0;
  height: 100%;
}

.titlebar-controls {
  display: flex;
  align-items: stretch;
  flex-shrink: 0;
  height: 100%;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.titlebar-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 100%;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  -webkit-app-region: no-drag;
  app-region: no-drag;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.titlebar-btn:hover {
  background: var(--accent);
  color: var(--foreground);
}

.titlebar-btn--close:hover {
  background: #e81123;
  color: #fff;
}
</style>
