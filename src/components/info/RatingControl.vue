<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ImageGroup } from '../../types/image'

const props = defineProps<{
  image: ImageGroup | null
}>()

const rating = ref(0)
const loading = ref(false)

async function loadRating() {
  // Rating comes from EXIF, loaded by ExifDisplay; for now this manages the UI state
  rating.value = 0
}

async function setRating(value: number) {
  if (!props.image) return
  const filePath = props.image.jpgPath || props.image.rawPath
  if (!filePath) return
  loading.value = true
  try {
    await invoke('write_rating', { filePath, rating: value })
    rating.value = value
  } catch (e) {
    console.error('Failed to write rating:', e)
  } finally {
    loading.value = false
  }
}

watch(() => props.image, () => {
  loadRating()
})

defineExpose({ setRating, rating })
</script>

<template>
  <div class="rating-control">
    <h3 class="section-title">评级</h3>
    <div v-if="!image" class="rating-empty">未选择图像</div>
    <div v-else class="stars">
      <span
        v-for="i in 5"
        :key="i"
        class="star"
        :class="{ active: i <= rating }"
        @click="setRating(i === rating ? 0 : i)"
      >
        &#9733;
      </span>
      <span class="rating-label">{{ rating > 0 ? rating + ' 星' : '未评级' }}</span>
    </div>
  </div>
</template>

<style scoped>
.rating-control {
  margin-top: var(--spacing-lg);
}

.section-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  margin-bottom: var(--spacing-md);
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border-light);
}

.rating-empty {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.stars {
  display: flex;
  align-items: center;
  gap: 4px;
}

.star {
  font-size: 22px;
  cursor: pointer;
  color: var(--color-star-empty);
  transition: color var(--transition-fast);
}

.star.active {
  color: var(--color-star);
}

.star:hover {
  color: var(--color-star);
}

.rating-label {
  margin-left: var(--spacing-sm);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}
</style>
