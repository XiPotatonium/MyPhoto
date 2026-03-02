<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'

export interface MenuItemDef {
  label: string
  action: () => void
}

const props = defineProps<{
  visible: boolean
  x: number
  y: number
  items: MenuItemDef[]
}>()

const emit = defineEmits<{
  close: []
}>()

const menuRef = ref<HTMLElement | null>(null)

function onClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit('close')
  }
}

function onItemClick(item: MenuItemDef) {
  item.action()
  emit('close')
}

// Adjust position to keep menu within viewport
const adjustedPosition = ref({ x: props.x, y: props.y })

watch(() => [props.x, props.y, props.visible], () => {
  if (!props.visible) return

  const menuWidth = 180
  const menuHeight = Math.min(props.items.length * 36 + 8, 300)

  let newX = props.x
  let newY = props.y

  // Keep within viewport horizontally
  if (newX + menuWidth > window.innerWidth) {
    newX = window.innerWidth - menuWidth - 8
  }

  // Keep within viewport vertically
  if (newY + menuHeight > window.innerHeight) {
    newY = window.innerHeight - menuHeight - 8
  }

  adjustedPosition.value = { x: Math.max(8, newX), y: Math.max(8, newY) }
}, { immediate: true })

onMounted(() => {
  document.addEventListener('mousedown', onClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', onClickOutside)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="menu">
      <div
        v-if="visible"
        ref="menuRef"
        class="context-menu"
        :style="{ left: adjustedPosition.x + 'px', top: adjustedPosition.y + 'px' }"
      >
        <div
          v-for="(item, i) in items"
          :key="i"
          class="context-menu-item"
          @click="onItemClick(item)"
        >
          {{ item.label }}
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  min-width: 160px;
  background: var(--popover);
  border: 1px solid var(--border);
  border-radius: calc(var(--radius) - 2px);
  box-shadow: var(--shadow-lg);
  padding: var(--spacing-xs);
  z-index: 9999;
}

.context-menu-item {
  padding: var(--spacing-sm) var(--spacing-md);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  border-radius: calc(var(--radius) - 4px);
  color: var(--popover-foreground);
}

.context-menu-item:hover {
  background: var(--accent);
  color: var(--accent-foreground);
}

/* Transition animations */
.menu-enter-active,
.menu-leave-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

.menu-enter-to,
.menu-leave-from {
  opacity: 1;
  transform: scale(1);
}
</style>
