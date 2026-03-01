<script setup lang="ts">
import { FileImage, FileType2 } from 'lucide-vue-next'
import type { ImageGroup } from '../../types/image'
import { Button } from '../ui/button'
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
    <div class="format-tabs">
      <Button
        v-if="image.jpgPath"
        variant="ghost"
        size="sm"
        :class="cn(
          'format-tab gap-1.5',
          currentFormat === 'jpg' && 'active'
        )"
        @click="emit('update:format', 'jpg')"
      >
        <FileImage class="h-4 w-4" />
        JPG
      </Button>
      <Button
        v-if="image.rawPath"
        variant="ghost"
        size="sm"
        :class="cn(
          'format-tab gap-1.5',
          currentFormat === 'raw' && 'active'
        )"
        @click="emit('update:format', 'raw')"
      >
        <FileType2 class="h-4 w-4" />
        RAW
      </Button>
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

.format-tabs {
  display: flex;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs);
  background: hsl(var(--muted));
  border-radius: var(--radius);
}

.format-tab {
  font-weight: 500;
  transition: all var(--transition-fast);
}

.format-tab.active {
  background: hsl(var(--background));
  color: hsl(var(--foreground));
  box-shadow: var(--shadow-sm);
}
</style>
