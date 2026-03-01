<script setup lang="ts">
import { useTheme } from '../../composables/useTheme'
import type { SortField, SortOrder } from '../../types/image'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select'
import { Button } from '../ui/button'
import { Sun, Moon, ArrowUp, ArrowDown } from 'lucide-vue-next'

defineProps<{
  sortField: SortField
  sortOrder: SortOrder
}>()

const emit = defineEmits<{
  'update:sortField': [field: SortField]
  'update:sortOrder': [order: SortOrder]
}>()

const { resolvedTheme, toggleTheme } = useTheme()

const sortOptions = [
  { value: 'name', label: '文件名' },
  { value: 'date', label: '拍摄时间' },
  { value: 'rating', label: '星级评分' },
] as const
</script>

<template>
  <div class="top-filter-bar">
    <div class="filter-section">
      <span class="filter-label">排序</span>
      <Select
        :model-value="sortField"
        @update:model-value="emit('update:sortField', $event as SortField)"
      >
        <SelectTrigger class="w-[140px] h-8">
          <SelectValue placeholder="选择排序方式" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem
            v-for="opt in sortOptions"
            :key="opt.value"
            :value="opt.value"
          >
            {{ opt.label }}
          </SelectItem>
        </SelectContent>
      </Select>
      <Button
        variant="secondary"
        size="icon"
        class="h-8 w-8"
        :title="sortOrder === 'asc' ? '升序' : '降序'"
        @click="emit('update:sortOrder', sortOrder === 'asc' ? 'desc' : 'asc')"
      >
        <ArrowUp v-if="sortOrder === 'asc'" class="h-4 w-4" />
        <ArrowDown v-else class="h-4 w-4" />
      </Button>
    </div>

    <div class="actions-section">
      <Button
        variant="ghost"
        size="icon"
        class="h-8 w-8"
        :title="resolvedTheme === 'light' ? '切换到深色模式' : '切换到浅色模式'"
        @click="toggleTheme"
      >
        <Sun v-if="resolvedTheme === 'light'" class="h-4 w-4" />
        <Moon v-else class="h-4 w-4" />
      </Button>
    </div>
  </div>
</template>

<style scoped>
.top-filter-bar {
  height: var(--topbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--spacing-lg);
  border-bottom: 1px solid hsl(var(--border));
  background: hsl(var(--background));
  flex-shrink: 0;
}

.filter-section {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.filter-label {
  font-size: var(--font-size-sm);
  color: hsl(var(--muted-foreground));
  font-weight: 500;
}

.actions-section {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}
</style>
