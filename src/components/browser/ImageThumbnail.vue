<script setup lang="ts">
import { ImageOff } from 'lucide-vue-next'
import type { ImageGroup } from '../../types/image'
import { cn } from '../../lib/utils'

defineProps<{
  image: ImageGroup
  thumbnail: string | null
  selected: boolean
}>()

const emit = defineEmits<{
  click: [e: MouseEvent]
}>()
</script>

<template>
  <div
    :class="cn(
      'image-thumbnail group',
      selected && 'selected'
    )"
    @click="(e) => emit('click', e)"
  >
    <div class="thumb-container">
      <img
        v-if="thumbnail"
        :src="'data:image/jpeg;base64,' + thumbnail"
        class="thumb-img"
        draggable="false"
      />
      <div v-else class="thumb-placeholder">
        <ImageOff class="h-8 w-8 text-muted-foreground/50" />
      </div>
      <div v-if="image.fileCount > 1" class="file-badge">
        {{ image.fileCount }}
      </div>
    </div>
    <div class="thumb-label" :title="image.baseName">
      {{ image.baseName }}
    </div>
  </div>
</template>

<style scoped>
.image-thumbnail {
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: pointer;
  padding: var(--spacing-xs);
  border-radius: calc(var(--radius) - 2px);
  transition: all var(--transition-fast);
  border: 2px solid transparent;
}

.image-thumbnail:hover {
  background: var(--accent);
}

.image-thumbnail.selected {
  background: var(--accent);
  border-color: var(--primary);
}

.thumb-container {
  position: relative;
  width: var(--thumbnail-size);
  height: var(--thumbnail-size);
  border-radius: calc(var(--radius) - 4px);
  overflow: hidden;
  background: var(--muted);
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.thumb-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--muted);
  color: var(--muted-foreground);
}

.file-badge {
  position: absolute;
  top: 4px;
  right: 4px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: var(--primary);
  color: var(--primary-foreground);
  font-size: 10px;
  font-weight: 600;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--shadow-sm);
}

.thumb-label {
  margin-top: var(--spacing-xs);
  font-size: var(--font-size-xs);
  color: var(--muted-foreground);
  text-align: center;
  width: var(--thumbnail-size);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.image-thumbnail.selected .thumb-label {
  color: var(--foreground);
  font-weight: 600;
}
</style>
