<script setup lang="ts">
import type { ImageGroup } from '../../types/image'

defineProps<{
  image: ImageGroup
  currentFormat: 'jpg' | 'raw'
}>()

const emit = defineEmits<{
  'update:format': [format: 'jpg' | 'raw']
}>()
</script>

<template>
  <div class="format-tab-bar">
    <button
      v-if="image.rawPath"
      class="format-tab"
      :class="{ active: currentFormat === 'raw' }"
      @click="emit('update:format', 'raw')"
    >
      RAW
    </button>
    <button
      v-if="image.jpgPath"
      class="format-tab"
      :class="{ active: currentFormat === 'jpg' }"
      @click="emit('update:format', 'jpg')"
    >
      JPG
    </button>
  </div>
</template>

<style scoped>
.format-tab-bar {
  display: flex;
  justify-content: center;
  padding: var(--spacing-sm);
  gap: var(--spacing-xs);
  border-bottom: 1px solid var(--color-border-light);
  flex-shrink: 0;
}

.format-tab {
  padding: 4px 16px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  font-weight: 500;
  transition: all var(--transition-fast);
  background: var(--color-bg-primary);
}

.format-tab:hover {
  border-color: var(--color-accent);
}

.format-tab.active {
  background: var(--color-accent);
  color: white;
  border-color: var(--color-accent);
}
</style>
