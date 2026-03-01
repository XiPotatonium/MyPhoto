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
    <div class="tab-container">
      <button
        v-if="image.jpgPath"
        class="format-tab"
        :class="{ active: currentFormat === 'jpg' }"
        @click="emit('update:format', 'jpg')"
      >
        <span class="tab-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <path d="M8 12h8M12 8v8"/>
          </svg>
        </span>
        <span class="tab-text">JPG</span>
        <span v-if="image.jpgPath && image.rawPath" class="tab-indicator"></span>
      </button>
      <button
        v-if="image.rawPath"
        class="format-tab"
        :class="{ active: currentFormat === 'raw' }"
        @click="emit('update:format', 'raw')"
      >
        <span class="tab-icon raw">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <path d="M7 12h2m2 0h2m2 0h2"/>
          </svg>
        </span>
        <span class="tab-text">RAW</span>
        <span v-if="image.jpgPath && image.rawPath" class="tab-indicator"></span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.format-tab-bar {
  display: flex;
  justify-content: center;
  padding: var(--spacing-3) var(--spacing-4);
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.tab-container {
  display: flex;
  gap: var(--spacing-1);
  padding: var(--spacing-1);
  background: var(--color-bg-secondary);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border);
}

.format-tab {
  display: flex;
  align-items: center;
  gap: var(--spacing-1);
  padding: var(--spacing-1) var(--spacing-3);
  border: none;
  border-radius: var(--radius-md);
  font-size: var(--font-size-xs);
  font-weight: var(--font-weight-semibold);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
  background: transparent;
  transition: all var(--transition-fast);
  position: relative;
}

.format-tab:hover {
  color: var(--color-text-primary);
  background: var(--color-bg-hover);
}

.format-tab.active {
  color: var(--color-accent);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-sm);
}

.tab-icon {
  width: 14px;
  height: 14px;
  opacity: 0.7;
}

.tab-icon svg {
  width: 100%;
  height: 100%;
}

.format-tab.active .tab-icon {
  opacity: 1;
}

.tab-icon.raw {
  color: var(--color-text-muted);
}

.format-tab.active .tab-icon.raw {
  color: var(--color-accent);
}

.tab-text {
  position: relative;
}

.tab-indicator {
  position: absolute;
  bottom: 2px;
  left: 50%;
  transform: translateX(-50%);
  width: 4px;
  height: 4px;
  background: var(--color-accent);
  border-radius: 50%;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.format-tab.active .tab-indicator {
  opacity: 1;
}
</style>
