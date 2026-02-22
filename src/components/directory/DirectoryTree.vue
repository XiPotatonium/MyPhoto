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
    <div v-if="!rootPath" class="tree-empty" @click="selectRootDirectory">
      点击此处设置根目录
    </div>
    <div v-else-if="loading" class="tree-loading">加载中...</div>
    <div v-else-if="tree" class="tree-content">
      <DirectoryTreeNode
        v-for="child in tree.children"
        :key="child.path"
        :node="child"
        :level="0"
        :selected-path="selectedPath"
        :expanded-paths="expandedPaths"
        @select="onFolderSelected"
        @toggle="toggleExpand"
      />
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
  padding: var(--spacing-xs) 0;
}

.tree-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: var(--font-size-sm);
}

.tree-empty:hover {
  color: var(--color-accent);
}

.tree-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}
</style>
