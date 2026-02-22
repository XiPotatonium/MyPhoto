<script setup lang="ts">
import type { DirectoryNode } from '../../types/directory'

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
      class="tree-node"
      :class="{ selected: selectedPath === node.path }"
      :style="{ paddingLeft: (level * 16 + 8) + 'px' }"
      @click="onClick"
    >
      <span class="tree-arrow" :class="{ expanded: isExpanded(), hidden: !hasChildren() }">
        &#9654;
      </span>
      <span class="tree-icon">&#128193;</span>
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
  padding: 3px 8px;
  cursor: pointer;
  white-space: nowrap;
  font-size: var(--font-size-sm);
  transition: background var(--transition-fast);
}

.tree-node:hover {
  background: var(--color-bg-hover);
}

.tree-node.selected {
  background: var(--color-accent-light);
  color: var(--color-accent);
}

.tree-arrow {
  font-size: 8px;
  width: 12px;
  text-align: center;
  transition: transform var(--transition-fast);
  flex-shrink: 0;
  color: var(--color-text-muted);
}

.tree-arrow.expanded {
  transform: rotate(90deg);
}

.tree-arrow.hidden {
  visibility: hidden;
}

.tree-icon {
  font-size: var(--font-size-base);
  flex-shrink: 0;
}

.tree-label {
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
