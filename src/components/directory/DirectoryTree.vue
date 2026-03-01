<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DirectoryNode } from '../../types/directory'
import DirectoryTreeNode from './DirectoryTreeNode.vue'
import ContextMenu from '../common/ContextMenu.vue'
import { useContextMenu } from '../../composables/useContextMenu'

const props = defineProps<{
  rootPath: string | null
}>()

const emit = defineEmits<{
  'folder-selected': [path: string]
  'root-changed': [path: string]
}>()

const tree = ref<DirectoryNode | null>(null)
const loading = ref(false)
const selectedPath = ref<string | null>(null)
const { menuState, showMenu, hideMenu } = useContextMenu()

async function loadTree(rootPath: string) {
  loading.value = true
  try {
    tree.value = await invoke<DirectoryNode>('scan_directory_tree', { rootPath })
  } catch (e) {
    console.error('Failed to scan directory tree:', e)
    tree.value = null
  } finally {
    loading.value = false
  }
}

async function selectRootDirectory() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: false })
    if (selected) {
      emit('root-changed', selected as string)
    }
  } catch (e) {
    console.error('Failed to open folder dialog:', e)
  }
}

function onFolderSelected(path: string) {
  selectedPath.value = path
  emit('folder-selected', path)
}

function onContextMenu(e: MouseEvent) {
  showMenu(e, [
    { label: '设置根目录', action: selectRootDirectory },
    { label: '刷新目录树', action: () => { if (props.rootPath) loadTree(props.rootPath) } },
    { label: '全部折叠', action: collapseAll },
  ])
}

function collapseAll() {
  expandedPaths.value.clear()
}

const expandedPaths = ref<Set<string>>(new Set())

function toggleExpand(path: string) {
  if (expandedPaths.value.has(path)) {
    expandedPaths.value.delete(path)
  } else {
    expandedPaths.value.add(path)
  }
}

watch(() => props.rootPath, (newPath) => {
  if (newPath) {
    loadTree(newPath)
  }
}, { immediate: true })
</script>

<template>
  <div class="directory-tree" @contextmenu.prevent="onContextMenu">
    <!-- Empty State -->
    <div v-if="!rootPath" class="tree-state empty" @click="selectRootDirectory">
      <div class="state-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          <line x1="12" y1="11" x2="12" y2="17"/>
          <line x1="9" y1="14" x2="15" y2="14"/>
        </svg>
      </div>
      <div class="state-title">设置根目录</div>
      <div class="state-desc">点击选择照片文件夹</div>
    </div>
    
    <!-- Loading State -->
    <div v-else-if="loading" class="tree-state loading">
      <div class="loading-spinner">
        <div class="spinner-dot"></div>
        <div class="spinner-dot"></div>
        <div class="spinner-dot"></div>
      </div>
      <div class="state-text">扫描文件夹...</div>
    </div>
    
    <!-- Tree Content -->
    <div v-else-if="tree" class="tree-content">
      <div class="tree-header">
        <span class="header-icon">📁</span>
        <span class="header-title" :title="rootPath">{{ tree.name }}</span>
      </div>
      <div class="tree-body">
        <DirectoryTreeNode
          :node="tree"
          :level="0"
          :selected-path="selectedPath"
          :expanded-paths="expandedPaths"
          @select="onFolderSelected"
          @toggle="toggleExpand"
        />
      </div>
    </div>
    
    <ContextMenu
      :visible="menuState.visible"
      :x="menuState.x"
      :y="menuState.y"
      :items="menuState.items"
      @close="hideMenu"
    />
  </div>
</template>

<style scoped>
.directory-tree {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  background: var(--color-bg-secondary);
}

/* State Screens */
.tree-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-6);
  text-align: center;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.tree-state.empty:hover {
  background: var(--color-bg-hover);
}

.state-icon {
  width: 48px;
  height: 48px;
  color: var(--color-text-muted);
  margin-bottom: var(--spacing-3);
  opacity: 0.5;
  transition: all var(--transition-fast);
}

.tree-state.empty:hover .state-icon {
  opacity: 0.8;
  color: var(--color-accent);
  transform: scale(1.1);
}

.state-icon svg {
  width: 100%;
  height: 100%;
}

.state-title {
  font-size: var(--font-size-md);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-1);
}

.state-desc {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

/* Loading Animation */
.loading-spinner {
  display: flex;
  gap: 4px;
  margin-bottom: var(--spacing-3);
}

.spinner-dot {
  width: 8px;
  height: 8px;
  background: var(--color-accent);
  border-radius: 50%;
  animation: bounce 1.4s ease-in-out infinite both;
}

.spinner-dot:nth-child(1) {
  animation-delay: -0.32s;
}

.spinner-dot:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%, 80%, 100% { transform: scale(0); }
  40% { transform: scale(1); }
}

.state-text {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

/* Tree Content */
.tree-content {
  padding: var(--spacing-2) 0;
}

.tree-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-3);
  margin: 0 var(--spacing-2) var(--spacing-2);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-xs);
}

.header-icon {
  font-size: var(--font-size-md);
  flex-shrink: 0;
}

.header-title {
  flex: 1;
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tree-body {
  padding: var(--spacing-1) 0;
}
</style>
