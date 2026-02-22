<script setup lang="ts">
import { ref, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{
  direction?: 'horizontal' | 'vertical'
  initialRatio?: number
  minSize?: number
}>(), {
  direction: 'vertical',
  initialRatio: 50,
  minSize: 80,
})

const containerRef = ref<HTMLElement | null>(null)
const ratio = ref(props.initialRatio)
const isDragging = ref(false)

function startDrag(e: MouseEvent) {
  e.preventDefault()
  isDragging.value = true
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value || !containerRef.value) return
  const rect = containerRef.value.getBoundingClientRect()
  let newRatio: number
  if (props.direction === 'vertical') {
    newRatio = ((e.clientY - rect.top) / rect.height) * 100
  } else {
    newRatio = ((e.clientX - rect.left) / rect.width) * 100
  }
  const minPercent = (props.minSize / (props.direction === 'vertical' ? rect.height : rect.width)) * 100
  ratio.value = Math.max(minPercent, Math.min(100 - minPercent, newRatio))
}

function stopDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

onUnmounted(() => {
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
})
</script>

<template>
  <div
    ref="containerRef"
    class="split-pane"
    :class="[direction, { dragging: isDragging }]"
  >
    <div class="split-pane-first" :style="{ flexBasis: ratio + '%' }">
      <slot name="first" />
    </div>
    <div class="split-pane-handle" @mousedown="startDrag">
      <div class="handle-bar" />
    </div>
    <div class="split-pane-second" :style="{ flexBasis: (100 - ratio) + '%' }">
      <slot name="second" />
    </div>
  </div>
</template>

<style scoped>
.split-pane {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.split-pane.vertical {
  flex-direction: column;
}

.split-pane.horizontal {
  flex-direction: row;
}

.split-pane.dragging {
  cursor: ns-resize;
}

.split-pane.horizontal.dragging {
  cursor: ew-resize;
}

.split-pane-first,
.split-pane-second {
  overflow: hidden;
  min-width: 0;
  min-height: 0;
}

.split-pane-handle {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-border-light);
  transition: background var(--transition-fast);
  z-index: 1;
}

.vertical > .split-pane-handle {
  height: var(--splitter-size);
  cursor: ns-resize;
}

.horizontal > .split-pane-handle {
  width: var(--splitter-size);
  cursor: ew-resize;
}

.split-pane-handle:hover,
.dragging > .split-pane-handle {
  background: var(--color-accent);
}

.handle-bar {
  border-radius: 1px;
}

.vertical .handle-bar {
  width: 32px;
  height: 2px;
  background: var(--color-border);
}

.horizontal .handle-bar {
  width: 2px;
  height: 32px;
  background: var(--color-border);
}

.split-pane-handle:hover .handle-bar,
.dragging > .split-pane-handle .handle-bar {
  background: transparent;
}
</style>
