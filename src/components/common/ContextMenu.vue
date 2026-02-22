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
        {{ item.label }}
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  min-width: 140px;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  padding: var(--spacing-xs) 0;
  z-index: 9999;
}

.context-menu-item {
  padding: 6px 16px;
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.context-menu-item:hover {
  background: var(--color-accent);
  color: white;
}
</style>
