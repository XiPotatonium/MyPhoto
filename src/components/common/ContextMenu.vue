<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

export interface MenuItemDef {
  label: string
  action: () => void
}

defineProps<{
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
        :style="{ left: x + 'px', top: y + 'px' }"
      >
        <div
          v-for="(item, i) in items"
          :key="i"
          class="context-menu-item"
          @click="onItemClick(item)"
        >
          <span class="item-text">{{ item.label }}</span>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  min-width: 160px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl), 0 0 0 1px rgba(0, 0, 0, 0.05);
  padding: var(--spacing-1) 0;
  z-index: 9999;
  backdrop-filter: blur(8px);
}

.context-menu-item {
  display: flex;
  align-items: center;
  padding: var(--spacing-2) var(--spacing-4);
  margin: 0 var(--spacing-1);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--color-text-primary);
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.context-menu-item:hover {
  background: var(--color-accent-alpha);
  color: var(--color-accent);
}

.context-menu-item:active {
  transform: scale(0.98);
}

.item-text {
  flex: 1;
}

/* Menu animation */
.menu-enter-active,
.menu-leave-active {
  transition: all var(--transition-fast);
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
  transform: scale(0.96) translateY(-4px);
}
</style>
