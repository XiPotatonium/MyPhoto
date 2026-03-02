<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { FolderOpen, RefreshCw } from 'lucide-vue-next'
import type { DirectoryNode } from '../../types/directory'
import DirectoryTreeNode from './DirectoryTreeNode.vue'
import ContextMenu from '../common/ContextMenu.vue'
import { useContextMenu } from '../../composables/useContextMenu'
import { Button } from '../ui/button'

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
  ])
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
    <div v-if="!rootPath" class="tree-empty-state">
      <Button variant="outline" class="gap-2" @click="selectRootDirectory">
        <FolderOpen class="h-4 w-4" />
        选择文件夹
      </Button>
    </div>
    <template v-else>
      <div v-if="loading" class="tree-loading">
        <RefreshCw class="h-5 w-5 animate-spin text-muted-foreground" />
        <span>加载中...</span>
      </div>
      <div v-else-if="tree" class="tree-content">
        <DirectoryTreeNode
          :node="tree"
          :level="0"
          :selected-path="selectedPath"
          :expanded-paths="expandedPaths"
          @select="onFolderSelected"
          @toggle="toggleExpand"
        />
      </div>
    </template>
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
  display: flex;
  flex-direction: column;
}

.tree-empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-lg);
}

.tree-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-xs) 0;
}

.tree-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-sm);
  height: 100%;
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
}
</style>
