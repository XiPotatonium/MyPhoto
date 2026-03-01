<script setup lang="ts">
import type { ImageGroup } from '../../types/image'
import { Button } from '../ui/button'
import { ButtonGroup } from '../ui/button-group'
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
    <ButtonGroup>
      <Button
        v-if="image.jpgPath"
        variant="ghost"
        size="sm"
        :class="cn(
          'format-tab',
          currentFormat === 'jpg' && 'active'
        )"
        @click="emit('update:format', 'jpg')"
      >
        JPG
      </Button>
      <Button
        v-if="image.rawPath"
        variant="ghost"
        size="sm"
        :class="cn(
          'format-tab',
          currentFormat === 'raw' && 'active'
        )"
        @click="emit('update:format', 'raw')"
      >
        RAW
      </Button>
    </ButtonGroup>
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
