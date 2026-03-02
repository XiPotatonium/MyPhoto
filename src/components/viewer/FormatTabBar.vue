<script setup lang="ts">
import type { ImageGroup } from '../../types/image'
import { cn } from '../../lib/utils'

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
    <div class="format-tab-group">
      <button
        v-if="image.jpgPath"
        :class="cn(
          'format-tab',
          currentFormat === 'jpg' && 'format-tab--active'
        )"
        @click="emit('update:format', 'jpg')"
      >
        JPG
      </button>
      <button
        v-if="image.rawPath"
        :class="cn(
          'format-tab',
          currentFormat === 'raw' && 'format-tab--active'
        )"
        @click="emit('update:format', 'raw')"
      >
        RAW
      </button>
    </div>
  </div>
</template>

<style scoped>
.format-tab-bar {
  display: flex;
  justify-content: center;
  padding: var(--spacing-sm);
  border-bottom: 1px solid hsl(var(--border));
  flex-shrink: 0;
  background: hsl(var(--background));
}

.format-tab-group {
  display: inline-flex;
  background: hsl(var(--muted));
  border-radius: var(--radius-md);
  padding: 3px;
  gap: 3px;
}

.format-tab {
  min-width: 72px;
  height: 32px;
  padding: 0 20px;
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: hsl(var(--muted-foreground));
  background: transparent;
  border: none;
  border-radius: calc(var(--radius-md) - 2px);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.format-tab:hover {
  color: hsl(var(--foreground));
}

.format-tab--active {
  background: hsl(var(--primary));
  color: hsl(var(--primary-foreground));
  box-shadow: var(--shadow-sm);
}

.format-tab--active:hover {
  background: hsl(var(--primary) / 0.9);
  color: hsl(var(--primary-foreground));
}
</style>
