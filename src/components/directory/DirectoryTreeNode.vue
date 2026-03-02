<script setup lang="ts">
import { ChevronRight, Folder, FolderOpen } from 'lucide-vue-next'
import type { DirectoryNode } from '../../types/directory'
import { cn } from '../../lib/utils'

const props = defineProps<{
  node: DirectoryNode
  level: number
  selectedPath: string | null
  expandedPaths: Set<string>
}>()

const emit = defineEmits<{
  select: [path: string]
  toggle: [path: string]
}>()

const isExpanded = () => props.expandedPaths.has(props.node.path)
const hasChildren = () => props.node.children && props.node.children.length > 0

function onClick() {
  emit('select', props.node.path)
  if (hasChildren()) {
    emit('toggle', props.node.path)
  }
}
</script>

<template>
  <div class="tree-node-wrapper">
    <div
      :class="cn(
        'tree-node',
        selectedPath === node.path && 'selected'
      )"
      :style="{ paddingLeft: `calc(${level} * var(--tree-indent) + var(--spacing-sm))` }"
      @click="onClick"
    >
      <ChevronRight
        :class="cn(
          'tree-arrow h-4 w-4 shrink-0 text-muted-foreground transition-transform duration-150',
          isExpanded() && 'rotate-90',
          !hasChildren() && 'invisible'
        )"
      />
      <component
        :is="isExpanded() ? FolderOpen : Folder"
        class="tree-icon h-4 w-4 shrink-0 text-primary"
      />
      <span class="tree-label">{{ node.name }}</span>
    </div>
    <div v-if="isExpanded() && hasChildren()" class="tree-children">
      <DirectoryTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :level="level + 1"
        :selected-path="selectedPath"
        :expanded-paths="expandedPaths"
        @select="(p: string) => emit('select', p)"
        @toggle="(p: string) => emit('toggle', p)"
      />
    </div>
  </div>
</template>

<style scoped>
.tree-node {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  cursor: pointer;
  white-space: nowrap;
  font-size: var(--font-size-sm);
  border-radius: calc(var(--radius) - 4px);
  margin: 0 var(--spacing-xs);
  transition: all var(--transition-fast);
}

.tree-node:hover {
  background: var(--accent);
}

.tree-node.selected {
  background: var(--primary);
  color: var(--primary-foreground);
}

.tree-node.selected .tree-icon {
  color: var(--primary-foreground);
}

.tree-label {
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}
</style>
