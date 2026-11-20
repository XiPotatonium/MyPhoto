<script setup lang="ts">
import { computed } from 'vue'
import { FolderClosed } from 'lucide-vue-next'
import type { FolderItem } from '../../types/folder'
import { cn } from '../../lib/utils'

const props = defineProps<{
  folder: FolderItem
  thumbnail: string | null
  size?: 'normal' | 'large'
}>()

// 文件夹不参与选中与框选，不要透传 data-index
defineOptions({ inheritAttrs: false })

const emit = defineEmits<{
  dblclick: [e: MouseEvent]
}>()

const coverUrl = computed(() =>
  props.thumbnail ? `data:image/jpeg;base64,${props.thumbnail}` : null,
)

const title = computed(() => `${props.folder.name} · ${props.folder.photoCount} 张照片`)
</script>

<template>
  <div
    :class="cn('folder-thumbnail', size === 'large' && 'folder-thumbnail--large')"
    :title="title"
    @mousedown.stop="() => {}"
    @click.stop="() => {}"
    @dblclick.stop="(e) => emit('dblclick', e)"
  >
    <div class="folder-card">
      <!-- 封面完整铺满圆角卡片；文件夹身份交给左上角徽标，不再用色块伪造 tab -->
      <img v-if="coverUrl" :src="coverUrl" class="folder-photo" draggable="false" />
      <div v-else class="folder-empty">
        <FolderClosed class="folder-empty-icon" aria-hidden="true" />
      </div>
      <div class="folder-badge">
        <FolderClosed class="folder-badge-icon" aria-hidden="true" />
        <span>{{ folder.photoCount }}</span>
      </div>
    </div>
    <!-- 与 ImageThumbnail 的 .thumb-label 完全对齐，无需占位空 div -->
    <div class="folder-label" :title="folder.name">{{ folder.name }}</div>
  </div>
</template>

<style scoped>
.folder-thumbnail {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-xs);
  border: 2px solid transparent;
  border-radius: calc(var(--radius) - 2px);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.folder-thumbnail:hover {
  background: var(--accent);
}

.folder-thumbnail:hover .folder-photo {
  filter: brightness(1.06);
}

/* 沿用 Card.vue 的 rounded + border + shadow-sm 语言，与照片卡片同一套圆角 */
.folder-card {
  position: relative;
  width: var(--thumbnail-size);
  height: var(--thumbnail-size);
  border-radius: calc(var(--radius) - 4px);
  overflow: hidden;
  background: var(--muted);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-sm);
}

.folder-photo {
  position: absolute;
  inset: 0;
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: filter var(--transition-fast);
}

.folder-empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--muted);
  color: var(--muted-foreground);
}

.folder-empty-icon {
  width: 42%;
  height: 42%;
  max-width: 72px;
  max-height: 72px;
  stroke-width: 1.25;
  opacity: 0.4;
}

/* 左上角胶囊徽标：FolderClosed 图标 + 直属照片数，是文件夹与照片的唯一区分点 */
.folder-badge {
  position: absolute;
  top: 6px;
  left: 6px;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 22px;
  padding: 0 8px;
  border: 1px solid rgb(255 255 255 / 0.22);
  border-radius: 11px;
  background: rgb(0 0 0 / 0.55);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
  pointer-events: none;
  transition: background var(--transition-fast),
              border-color var(--transition-fast),
              color var(--transition-fast);
}

.folder-badge-icon {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
}

.folder-thumbnail:hover .folder-badge {
  background: var(--primary);
  border-color: var(--primary);
  color: var(--primary-foreground);
}

.folder-label {
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

/* Large size variant（0.3.2 弹性网格）：与 ImageThumbnail 同一套规则 */
.folder-thumbnail--large .folder-card {
  width: min(100%, var(--thumbnail-size-large));
  height: auto;
  aspect-ratio: 1 / 1;
}

.folder-thumbnail--large .folder-label {
  width: min(100%, var(--thumbnail-size-large));
}

.folder-thumbnail--large .folder-badge {
  top: 8px;
  left: 8px;
  height: 26px;
  padding: 0 10px;
  border-radius: 13px;
  font-size: 12px;
}

.folder-thumbnail--large .folder-badge-icon {
  width: 15px;
  height: 15px;
}
</style>
