<script setup lang="ts">
import type { SortField, SortOrder } from '../../types/image'

defineProps<{
  sortField: SortField
  sortOrder: SortOrder
}>()

const emit = defineEmits<{
  'update:sortField': [field: SortField]
  'update:sortOrder': [order: SortOrder]
}>()
</script>

<template>
  <div class="top-filter-bar">
    <div class="filter-group">
      <label class="filter-label">排序</label>
      <select
        class="filter-select"
        :value="sortField"
        @change="emit('update:sortField', ($event.target as HTMLSelectElement).value as SortField)"
      >
        <option value="name">按文件名</option>
        <option value="date">按时间</option>
        <option value="rating">按星级</option>
      </select>
      <button
        class="order-btn"
        :title="sortOrder === 'asc' ? '升序' : '降序'"
        @click="emit('update:sortOrder', sortOrder === 'asc' ? 'desc' : 'asc')"
      >
        {{ sortOrder === 'asc' ? '↑' : '↓' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.top-filter-bar {
  height: var(--topbar-height);
  display: flex;
  align-items: center;
  padding: 0 var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  flex-shrink: 0;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.filter-label {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.filter-select {
  padding: 3px 8px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg-primary);
  font-size: var(--font-size-sm);
  outline: none;
}

.filter-select:focus {
  border-color: var(--color-accent);
}

.order-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg-primary);
  font-size: var(--font-size-lg);
  transition: all var(--transition-fast);
}

.order-btn:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-accent);
}
</style>
