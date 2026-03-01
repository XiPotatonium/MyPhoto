<script setup lang="ts">
import type { SortField, SortOrder } from '../../types/image'
import type { Theme } from '../../stores/appState'

const props = defineProps<{
  sortField: SortField
  sortOrder: SortOrder
  theme: Theme
}>()

const emit = defineEmits<{
  'update:sortField': [field: SortField]
  'update:sortOrder': [order: SortOrder]
  'update:theme': [theme: Theme]
}>()

function toggleTheme() {
  const themes: Theme[] = ['light', 'dark', 'system']
  const currentIndex = themes.indexOf(props.theme)
  const nextTheme = themes[(currentIndex + 1) % themes.length]
  emit('update:theme', nextTheme)
}

function getThemeIcon(theme: Theme): string {
  switch (theme) {
    case 'light': return '☀️'
    case 'dark': return '🌙'
    case 'system': return '💻'
  }
}

function getThemeLabel(theme: Theme): string {
  switch (theme) {
    case 'light': return '浅色'
    case 'dark': return '深色'
    case 'system': return '跟随系统'
  }
}
</script>

<template>
  <div class="top-filter-bar">
    <div class="bar-left">
      <div class="app-brand">
        <span class="brand-icon">📷</span>
        <span class="brand-text">MyPhoto</span>
      </div>
    </div>
    
    <div class="bar-center">
      <div class="filter-group">
        <label class="filter-label">排序方式</label>
        <div class="filter-controls">
          <select
            class="filter-select"
            :value="sortField"
            @change="emit('update:sortField', ($event.target as HTMLSelectElement).value as SortField)"
          >
            <option value="name">文件名</option>
            <option value="date">拍摄时间</option>
            <option value="rating">星级评分</option>
          </select>
          <button
            class="order-btn"
            :class="{ 'desc': sortOrder === 'desc' }"
            :title="sortOrder === 'asc' ? '升序' : '降序'"
            @click="emit('update:sortOrder', sortOrder === 'asc' ? 'desc' : 'asc')"
          >
            <svg class="order-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 5v14M5 12l7-7 7 7" v-if="sortOrder === 'asc'"/>
              <path d="M12 19V5M5 12l7 7 7-7" v-else/>
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <div class="bar-right">
      <button
        class="theme-toggle"
        :title="`主题: ${getThemeLabel(theme)} (点击切换)`"
        @click="toggleTheme"
      >
        <span class="theme-icon">{{ getThemeIcon(theme) }}</span>
        <span class="theme-label">{{ getThemeLabel(theme) }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.top-filter-bar {
  height: var(--topbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--spacing-5);
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
  box-shadow: var(--shadow-xs);
}

/* Left Section - Brand */
.bar-left {
  display: flex;
  align-items: center;
  flex: 1;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  font-family: var(--font-family-display);
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-primary);
}

.brand-icon {
  font-size: var(--font-size-xl);
}

.brand-text {
  background: linear-gradient(135deg, var(--color-accent) 0%, var(--color-accent-hover) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* Center Section - Filters */
.bar-center {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-1) var(--spacing-3);
  background: var(--color-bg-secondary);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border);
}

.filter-label {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.filter-controls {
  display: flex;
  align-items: center;
  gap: var(--spacing-1);
}

.filter-select {
  padding: var(--spacing-1) var(--spacing-2);
  padding-right: var(--spacing-6);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--color-text-primary);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-md);
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23999' stroke-width='2'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  transition: all var(--transition-fast);
}

.filter-select:hover {
  border-color: var(--color-accent);
}

.order-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.order-btn:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.order-btn:active {
  transform: scale(0.95);
}

.order-icon {
  width: 14px;
  height: 14px;
}

/* Right Section - Theme Toggle */
.bar-right {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex: 1;
}

.theme-toggle {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-1) var(--spacing-3);
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.theme-toggle:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-accent);
  color: var(--color-text-primary);
}

.theme-toggle:active {
  transform: scale(0.98);
}

.theme-icon {
  font-size: var(--font-size-md);
}

.theme-label {
  min-width: 56px;
  text-align: center;
}
</style>
