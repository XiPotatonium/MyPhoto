<script setup lang="ts">
import type { ImageGroup } from '../../types/image'

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
    class="image-thumbnail"
    :class="{ selected }"
    @click="(e) => emit('click', e)"
  >
    <div class="thumb-container">
      <img
        v-if="thumbnail"
        :src="'data:image/jpeg;base64,' + thumbnail"
        class="thumb-img"
        draggable="false"
      />
      <div v-else class="thumb-placeholder" />
      <div v-if="image.fileCount > 1" class="file-badge">{{ image.fileCount }}</div>
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
  border-radius: var(--radius-md);
  transition: background var(--transition-fast);
}

.image-thumbnail:hover {
  background: var(--color-bg-hover);
}

.image-thumbnail.selected {
  background: var(--color-accent-light);
}

.thumb-container {
  position: relative;
  width: var(--thumbnail-size);
  height: var(--thumbnail-size);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--color-bg-tertiary);
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumb-placeholder {
  width: 100%;
  height: 100%;
  background: var(--color-bg-tertiary);
}

.file-badge {
  position: absolute;
  top: 4px;
  right: 4px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: var(--color-badge-bg);
  color: var(--color-badge-text);
  font-size: 10px;
  font-weight: 600;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.thumb-label {
  margin-top: var(--spacing-xs);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  text-align: center;
  width: var(--thumbnail-size);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
