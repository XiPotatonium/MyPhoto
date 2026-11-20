<script setup lang="ts">
import { ImageOff, Star, Info } from 'lucide-vue-next'
import type { ImageGroup } from '../../types/image'
import { cn } from '../../lib/utils'

defineProps<{
  image: ImageGroup
  thumbnail: string | null
  selected: boolean
  size?: 'normal' | 'large'
  // 当前右侧详情面板正在展示这张图：此时 i 按钮常显并高亮
  infoActive?: boolean
}>()

// 允许透传 data-index 等属性到根元素
defineOptions({ inheritAttrs: true })

const emit = defineEmits<{
  click: [e: MouseEvent]
  dblclick: [e: MouseEvent]
  'info-click': [image: ImageGroup]
}>()
</script>

<template>
  <div
    :class="cn(
      'image-thumbnail group',
      selected && 'selected',
      size === 'large' && 'image-thumbnail--large'
    )"
    @click="(e) => emit('click', e)"
    @dblclick="(e) => emit('dblclick', e)"
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
      <div v-if="image.exifInfo?.rating && image.exifInfo.rating > 0" class="rating-badge">
        <Star
          v-for="i in 5"
          :key="i"
          :class="cn(
            'rating-star',
            i <= (image.exifInfo?.rating || 0) ? 'rating-star--filled' : 'rating-star--empty'
          )"
        />
      </div>
      <!-- 0.3.0：hover 时出现的信息按钮，点击展开右侧详情页。
           阻止冒泡，避免误触发点选、双击进入大图与网格框选。 -->
      <button
        type="button"
        :class="cn('info-button', infoActive && 'info-button--active')"
        :title="infoActive ? '收起图片信息' : '查看图片信息'"
        :aria-label="infoActive ? '收起图片信息' : '查看图片信息'"
        @mousedown.stop="() => {}"
        @click.stop="emit('info-click', image)"
        @dblclick.stop="() => {}"
      >
        <Info class="info-icon" />
      </button>
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

/* Large size variant */
.image-thumbnail--large .thumb-container {
  width: var(--thumbnail-size-large);
  height: var(--thumbnail-size-large);
}

.image-thumbnail--large .thumb-label {
  width: var(--thumbnail-size-large);
}

/* Rating badge */
.rating-badge {
  position: absolute;
  bottom: 3px;
  left: 3px;
  display: flex;
  align-items: center;
  gap: 1px;
  padding: 1px 3px;
  background: oklch(from var(--foreground) l c h / 0.5);
  border-radius: 3px;
}

.rating-star {
  width: 10px;
  height: 10px;
  flex-shrink: 0;
}

.rating-star--filled {
  fill: var(--color-star);
  color: var(--color-star);
}

.rating-star--empty {
  fill: none;
  color: oklch(1 0 0 / 0.4);
}

.image-thumbnail--large .rating-star {
  width: 14px;
  height: 14px;
}

.image-thumbnail--large .rating-badge {
  padding: 2px 4px;
  gap: 1px;
  bottom: 4px;
  left: 4px;
  border-radius: 4px;
}

/* 0.3.0：右下角的图片信息按钮。默认隐藏，hover / 键盘聚焦 /
   当前详情面板正展示此图时显示。评分角标已让到左下角避免重叠。 */
.info-button {
  position: absolute;
  right: 4px;
  bottom: 4px;
  width: 24px;
  height: 24px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgb(255 255 255 / 0.35);
  border-radius: 50%;
  background: rgb(0 0 0 / 0.62);
  color: #fff;
  cursor: pointer;
  opacity: 0;
  transform: scale(0.85);
  pointer-events: none;
  z-index: 2;
  transition: opacity var(--transition-fast),
              transform var(--transition-fast),
              background var(--transition-fast),
              border-color var(--transition-fast);
}

.image-thumbnail:hover .info-button,
.info-button:focus-visible,
.info-button--active {
  opacity: 1;
  transform: scale(1);
  pointer-events: auto;
}

.info-button:hover,
.info-button--active {
  background: var(--primary);
  border-color: var(--primary);
  color: var(--primary-foreground);
}

.info-icon {
  width: 14px;
  height: 14px;
}

.image-thumbnail--large .info-button {
  right: 6px;
  bottom: 6px;
  width: 26px;
  height: 26px;
}
</style>
