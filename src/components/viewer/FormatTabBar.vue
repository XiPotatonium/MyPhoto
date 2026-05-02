<script setup lang="ts">
import type { ImageGroup } from '../../types/image'
import { cn } from '../../lib/utils'
import { Card, CardHeader, CardTitle, CardAction } from '../ui/card'

const props = withDefaults(defineProps<{
  image: ImageGroup
  currentFormat: 'jpg' | 'raw'
  disabled?: boolean
}>(), {
  disabled: false,
})

const emit = defineEmits<{
  'update:format': [format: 'jpg' | 'raw']
}>()

function switchFormat(format: 'jpg' | 'raw') {
  if (props.disabled) return
  emit('update:format', format)
}
</script>

<template>
  <Card :class="cn('format-card', disabled && 'format-card--disabled')">
    <CardHeader class="format-card-header">
      <CardTitle class="format-card-title">
        {{ image.baseName }}
      </CardTitle>
      <CardAction>
        <div class="format-tabs">
          <button
            v-if="image.jpgPath"
            :class="cn(
              'format-btn',
              currentFormat === 'jpg' && 'format-btn--active',
              disabled && 'format-btn--disabled'
            )"
            :disabled="disabled"
            @click="switchFormat('jpg')"
          >
            JPG
          </button>
          <button
            v-if="image.rawPath"
            :class="cn(
              'format-btn',
              currentFormat === 'raw' && 'format-btn--active',
              disabled && 'format-btn--disabled'
            )"
            :disabled="disabled"
            @click="switchFormat('raw')"
          >
            RAW
          </button>
        </div>
      </CardAction>
    </CardHeader>
  </Card>
</template>

<style scoped>
.format-card {
  /* Override Card default py-6 gap-6 for compact layout */
  padding-block: 0 !important;
  gap: 0 !important;
}

.format-card--disabled {
  opacity: 0.6;
}

.format-card-header {
  display: flex;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md) !important;
  gap: var(--spacing-sm);
}

.format-card-title {
  font-size: var(--font-size-sm) !important;
  font-weight: var(--font-weight-semibold) !important;
  color: var(--foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
  letter-spacing: var(--letter-spacing-tight);
}

.format-tabs {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  background: var(--muted);
  border-radius: var(--radius-sm);
  padding: 2px;
}

.format-btn {
  min-width: 48px;
  height: 26px;
  padding: 0 var(--spacing-sm);
  font-size: var(--font-size-xs);
  font-weight: var(--font-weight-medium);
  letter-spacing: var(--letter-spacing-wider);
  color: var(--muted-foreground);
  background: transparent;
  border: none;
  border-radius: calc(var(--radius-sm) - 2px);
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.format-btn:hover:not(.format-btn--active):not(:disabled) {
  color: var(--foreground);
  background: oklch(from var(--muted) l c h / 0.5);
}

.format-btn--active {
  background: var(--primary);
  color: var(--primary-foreground);
  box-shadow: var(--shadow-xs);
}

.format-btn--active:hover {
  background: var(--primary);
  color: var(--primary-foreground);
}

.format-btn--disabled {
  cursor: not-allowed;
  opacity: 0.7;
}
</style>
