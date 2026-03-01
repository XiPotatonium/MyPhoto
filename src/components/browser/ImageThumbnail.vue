<script setup lang="ts">
import { computed } from 'vue'
import type { ImageGroup } from '../../types/image'

const props = defineProps<{
  image: ImageGroup
  thumbnail: string | null
  selected: boolean
}>()

const emit = defineEmits<{
  click: [e: MouseEvent]
}>()

const hasRating = computed(() => {
  return props.image.exifInfo?.rating && props.image.exifInfo.rating > 0
})

const rating = computed(() => {
  return props.image.exifInfo?.rating || 0
})
</script>

<template>
  <div
    class="image-thumbnail"
    :class="{ selected }"
    @click="(e) => emit('click', e)"
  >
    <div class="thumb-wrapper">
      <div class="thumb-container">
        <img
          v-if="thumbnail"
          :src="'data:image/jpeg;base64,' + thumbnail"
          class="thumb-img"
          draggable="false"
          loading="lazy"
        />
        <div v-else class="thumb-placeholder">
          <div class="placeholder-spinner" />
        </div>
        
        <!-- File count badge -->
        <div v-if="image.fileCount > 1" class="file-badge">
          <span class="badge-icon">📁</span>
          <span class="badge-text">{{ image.fileCount }}</span>
        </div>
        
        <!-- Rating indicator -->
        <div v-if="hasRating" class="rating-indicator">
          <span class="rating-star">★</span>
          <span class="rating-value">{{ rating }}</span>
        </div>
        
        <!-- Hover overlay -->
        <div class="thumb-overlay">
          <div class="overlay-content">
            <span class="view-icon">👁</span>
          </div>
        </div>
      </div>
      
      <!-- Selection indicator -->
      <div class="selection-indicator">
        <div class="selection-check">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <path d="M20 6L9 17l-5-5"/>
          </svg>
        </div>
      </div>
    </div>
    
    <div class="thumb-info">
      <div class="thumb-label" :title="image.baseName">
        {{ image.baseName }}
      </div>
      <div v-if="image.exifInfo?.datetime" class="thumb-meta">
        {{ image.exifInfo.datetime.split(' ')[0] }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.image-thumbnail {
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: pointer;
  padding: var(--spacing-2);
  border-radius: var(--radius-lg);
  transition: all var(--transition-normal);
  position: relative;
}

.image-thumbnail:hover {
  background: var(--color-bg-hover);
  transform: translateY(-2px);
}

.image-thumbnail.selected {
  background: var(--color-accent-alpha);
}

.image-thumbnail.selected .thumb-container {
  box-shadow: 0 0 0 2px var(--color-accent), var(--shadow-md);
}

/* Thumb Wrapper */
.thumb-wrapper {
  position: relative;
  width: var(--thumbnail-size);
  height: var(--thumbnail-size);
}

/* Thumb Container */
.thumb-container {
  position: relative;
  width: 100%;
  height: 100%;
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--color-bg-tertiary);
  box-shadow: var(--shadow-sm);
  transition: all var(--transition-normal);
}

.image-thumbnail:hover .thumb-container {
  box-shadow: var(--shadow-md);
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform var(--transition-slow);
}

.image-thumbnail:hover .thumb-img {
  transform: scale(1.05);
}

/* Placeholder */
.thumb-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--color-bg-tertiary) 0%, var(--color-bg-hover) 100%);
}

.placeholder-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--color-border);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* File Badge */
.file-badge {
  position: absolute;
  top: var(--spacing-1);
  right: var(--spacing-1);
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px 6px;
  background: var(--color-accent);
  color: var(--color-text-inverse);
  font-size: var(--font-size-2xs);
  font-weight: var(--font-weight-semibold);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-sm);
}

.badge-icon {
  font-size: 8px;
}

/* Rating Indicator */
.rating-indicator {
  position: absolute;
  bottom: var(--spacing-1);
  left: var(--spacing-1);
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-2xs);
  font-weight: var(--font-weight-semibold);
}

.rating-star {
  color: var(--color-star);
}

.rating-value {
  color: white;
}

/* Hover Overlay */
.thumb-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-overlay);
  opacity: 0;
  transition: opacity var(--transition-normal);
}

.image-thumbnail:hover .thumb-overlay {
  opacity: 1;
}

.overlay-content {
  transform: translateY(8px);
  transition: transform var(--transition-spring);
}

.image-thumbnail:hover .overlay-content {
  transform: translateY(0);
}

.view-icon {
  font-size: var(--font-size-xl);
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

/* Selection Indicator */
.selection-indicator {
  position: absolute;
  top: -6px;
  right: -6px;
  width: 22px;
  height: 22px;
  background: var(--color-accent);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transform: scale(0.5);
  transition: all var(--transition-spring);
  box-shadow: var(--shadow-sm);
  z-index: 1;
}

.image-thumbnail.selected .selection-indicator {
  opacity: 1;
  transform: scale(1);
}

.selection-check {
  width: 12px;
  height: 12px;
  color: white;
}

/* Thumb Info */
.thumb-info {
  margin-top: var(--spacing-2);
  width: 100%;
  text-align: center;
}

.thumb-label {
  font-size: var(--font-size-xs);
  font-weight: var(--font-weight-medium);
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: color var(--transition-fast);
}

.image-thumbnail:hover .thumb-label {
  color: var(--color-accent);
}

.thumb-meta {
  margin-top: 2px;
  font-size: var(--font-size-2xs);
  color: var(--color-text-muted);
}
</style>
