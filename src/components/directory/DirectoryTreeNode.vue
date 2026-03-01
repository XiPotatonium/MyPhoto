<script setup lang="ts">
import { computed } from 'vue'
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

const isExpanded = computed(() => props.expandedPaths.has(props.node.path))
const hasChildren = computed(() => props.node.children && props.node.children.length > 0)
const isSelected = computed(() => props.selectedPath === props.node.path)

function onClick() {
  emit('select', props.node.path)
  if (hasChildren.value) {
    emit('toggle', props.node.path)
  }
}

function onArrowClick(e: MouseEvent) {
  e.stopPropagation()
  if (hasChildren.value) {
    emit('toggle', props.node.path)
  }
}
</script>

<template>
  <div class="tree-node-wrapper">
    <div
      class="tree-node"
      :class="{ 
        selected: isSelected,
        expanded: isExpanded,
        'has-children': hasChildren
      }"
      :style="{ paddingLeft: (level * 14 + 6) + 'px' }"
      @click="onClick"
    >
      <!-- Expand/Collapse Arrow -->
      <button 
        class="tree-arrow"
        :class="{ expanded: isExpanded, hidden: !hasChildren }"
        @click="onArrowClick"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M9 18l6-6-6-6"/>
        </svg>
      </button>
      
      <!-- Folder Icon -->
      <span class="tree-icon">
        <svg v-if="isExpanded" viewBox="0 0 24 24" fill="currentColor">
          <path d="M20 6h-8l-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2z"/>
        </svg>
        <svg v-else viewBox="0 0 24 24" fill="currentColor">
          <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
        </svg>
      </span>
      
      <!-- Folder Name -->
      <span class="tree-label">{{ node.name }}</span>
      
      <!-- Child Count Badge -->
      <span v-if="hasChildren && node.children" class="child-count">
        {{ node.children.length }}
      </span>
    </div>
    
    <!-- Children Container with Animation -->
    <div 
      v-show="isExpanded && hasChildren" 
      class="tree-children"
      :class="{ expanded: isExpanded }"
    >
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
.tree-node-wrapper {
  position: relative;
}

/* Connector Lines */
.tree-node-wrapper::before {
  content: '';
  position: absolute;
  left: v-bind('(level * 14 + 14) + "px"');
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--color-border);
  opacity: 0.5;
  pointer-events: none;
}

.tree-node {
  display: flex;
  align-items: center;
  gap: var(--spacing-1);
  padding: var(--spacing-1) var(--spacing-2);
  margin: 1px var(--spacing-1);
  cursor: pointer;
  white-space: nowrap;
  font-size: var(--font-size-sm);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
  position: relative;
}

.tree-node:hover {
  background: var(--color-bg-hover);
}

.tree-node.selected {
  background: var(--color-accent-alpha);
  color: var(--color-accent);
}

.tree-node.selected .tree-icon {
  color: var(--color-accent);
}

/* Expand/Collapse Arrow */
.tree-arrow {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.tree-arrow:hover {
  background: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}

.tree-arrow svg {
  width: 14px;
  height: 14px;
  transition: transform var(--transition-normal);
}

.tree-arrow.expanded svg {
  transform: rotate(90deg);
}

.tree-arrow.hidden {
  visibility: hidden;
  pointer-events: none;
}

/* Folder Icon */
.tree-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  color: var(--color-accent);
  opacity: 0.8;
  transition: all var(--transition-fast);
}

.tree-node:hover .tree-icon {
  opacity: 1;
}

.tree-icon svg {
  width: 100%;
  height: 100%;
}

/* Folder Label */
.tree-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: var(--font-weight-medium);
  color: var(--color-text-primary);
  transition: color var(--transition-fast);
}

.tree-node.selected .tree-label {
  font-weight: var(--font-weight-semibold);
}

/* Child Count Badge */
.child-count {
  font-size: var(--font-size-2xs);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-muted);
  background: var(--color-bg-tertiary);
  padding: 1px 5px;
  border-radius: var(--radius-full);
  min-width: 18px;
  text-align: center;
}

.tree-node.selected .child-count {
  background: var(--color-accent);
  color: var(--color-text-inverse);
}

/* Children Container */
.tree-children {
  position: relative;
  animation: slideDown var(--transition-normal) ease-out;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
