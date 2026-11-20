<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ArrowDown, ArrowUp, ArrowUpDown, Check, FolderOpen, Moon, Sun } from 'lucide-vue-next'
import type { SortField, SortOrder } from '../../types/image'
import PathBreadcrumb from '../browser/PathBreadcrumb.vue'

// 0.3.3：原生菜单栏移除后，打开目录 / 排序 / 主题全部并入这一行。
// 路径栏是 flex:1 的中间区域，左侧固定「打开目录」文件夹按钮，右侧固定排序与主题按钮，
// 因此 PathBreadcrumb 自己测到的宽度天然已扣掉两侧按钮，折叠逻辑无需改动。
const props = defineProps<{
  rootPath: string | null
  currentPath: string | null
  sortField: SortField
  sortOrder: SortOrder
  theme: 'light' | 'dark'
}>()

const emit = defineEmits<{
  navigate: [path: string]
  'open-folder': []
  'sort-field': [field: SortField]
  'sort-order': [order: SortOrder]
  'toggle-theme': []
}>()

const fields: { value: SortField; label: string }[] = [
  { value: 'name', label: '文件名' },
  { value: 'date', label: '拍摄时间' },
  { value: 'rating', label: '星级评分' },
]
const orders: { value: SortOrder; label: string }[] = [
  { value: 'asc', label: '升序' },
  { value: 'desc', label: '降序' },
]

const sortOpen = ref(false)
const sortBtn = ref<HTMLElement | null>(null)
const sortMenuRef = ref<HTMLElement | null>(null)
const menuStyle = ref<Record<string, string>>({})

function toggleSortMenu() {
  if (sortOpen.value) {
    sortOpen.value = false
    return
  }
  const rect = sortBtn.value?.getBoundingClientRect()
  if (!rect) return
  menuStyle.value = {
    right: `${Math.max(8, window.innerWidth - rect.right)}px`,
    top: `${rect.bottom + 6}px`,
  }
  sortOpen.value = true
}

function onDocMouseDown(e: MouseEvent) {
  if (!sortOpen.value) return
  const target = e.target as Node
  if (sortMenuRef.value?.contains(target) || sortBtn.value?.contains(target)) return
  sortOpen.value = false
}

// 捕获阶段拦截：下拉打开时 Esc 只关本菜单，不触发 App.vue 的全局 Esc
function onGlobalKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && sortOpen.value) {
    sortOpen.value = false
    e.preventDefault()
    e.stopPropagation()
  }
}

function chooseField(value: SortField) {
  emit('sort-field', value)
  sortOpen.value = false
}
function chooseOrder(value: SortOrder) {
  emit('sort-order', value)
  sortOpen.value = false
}

watch(sortOpen, open => {
  if (open) document.addEventListener('mousedown', onDocMouseDown)
  else document.removeEventListener('mousedown', onDocMouseDown)
})

function closeOnResize() {
  sortOpen.value = false
}
onMounted(() => {
  window.addEventListener('keydown', onGlobalKeydown, true)
  window.addEventListener('resize', closeOnResize)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onGlobalKeydown, true)
  window.removeEventListener('resize', closeOnResize)
  document.removeEventListener('mousedown', onDocMouseDown)
})

const sortTitle = computed(() => {
  const field = fields.find(f => f.value === props.sortField)?.label ?? '文件名'
  const order = props.sortOrder === 'asc' ? '升序' : '降序'
  return `排序：${field} · ${order}（点击更改）`
})
</script>

<template>
  <div class="top-toolbar">
    <button
      type="button"
      class="tool-btn tool-btn--folder"
      title="打开目录…"
      aria-label="打开目录"
      @click="emit('open-folder')"
    >
      <FolderOpen class="h-4 w-4" />
    </button>

    <div class="toolbar-path">
      <PathBreadcrumb
        :root-path="rootPath"
        :current-path="currentPath"
        @navigate="(p: string) => emit('navigate', p)"
      />
    </div>

    <div class="toolbar-actions">
      <button
        ref="sortBtn"
        type="button"
        class="tool-btn"
        :class="sortOpen && 'tool-btn--active'"
        :title="sortTitle"
        aria-label="排序"
        :aria-expanded="sortOpen"
        @click="toggleSortMenu"
      >
        <ArrowUpDown class="h-4 w-4" />
      </button>

      <button
        type="button"
        class="tool-btn"
        :title="theme === 'dark' ? '切换到浅色模式' : '切换到深色模式'"
        :aria-label="theme === 'dark' ? '切换到浅色模式' : '切换到深色模式'"
        @click="emit('toggle-theme')"
      >
        <Sun v-if="theme === 'dark'" class="h-4 w-4" />
        <Moon v-else class="h-4 w-4" />
      </button>
    </div>

    <Teleport to="body">
      <Transition name="sort-menu">
        <div v-if="sortOpen" ref="sortMenuRef" class="sort-menu" :style="menuStyle">
          <div class="sort-menu-caption">排序依据</div>
          <button
            v-for="opt in fields"
            :key="opt.value"
            type="button"
            class="sort-menu-item"
            :class="props.sortField === opt.value && 'sort-menu-item--active'"
            @click="chooseField(opt.value)"
          >
            <span>{{ opt.label }}</span>
            <Check v-if="props.sortField === opt.value" class="h-3.5 w-3.5" />
          </button>

          <div class="sort-menu-divider" />

          <div class="sort-menu-caption">顺序</div>
          <button
            v-for="opt in orders"
            :key="opt.value"
            type="button"
            class="sort-menu-item"
            :class="props.sortOrder === opt.value && 'sort-menu-item--active'"
            @click="chooseOrder(opt.value)"
          >
            <ArrowUp v-if="opt.value === 'asc'" class="h-3.5 w-3.5 sort-menu-hint" />
            <ArrowDown v-else class="h-3.5 w-3.5 sort-menu-hint" />
            <span>{{ opt.label }}</span>
            <Check v-if="props.sortOrder === opt.value" class="h-3.5 w-3.5" />
          </button>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.top-toolbar {
  flex: 0 0 44px;
  display: flex;
  align-items: center;
  gap: 6px;
  height: 44px;
  padding: 0 10px;
  border-bottom: 1px solid var(--border);
  background: transparent;
  user-select: none;
}

.toolbar-path {
  flex: 1 1 auto;
  min-width: 0;
  height: 100%;
}

.toolbar-actions {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  gap: 4px;
}

.tool-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  flex-shrink: 0;
  padding: 0;
  border: none;
  border-radius: calc(var(--radius) - 4px);
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.tool-btn:hover,
.tool-btn--active {
  background: var(--accent);
  color: var(--foreground);
}

.tool-btn--folder {
  color: var(--foreground);
}

.sort-menu {
  position: fixed;
  z-index: var(--z-popover);
  width: 184px;
  padding: var(--spacing-xs);
  border: 1px solid var(--border);
  border-radius: calc(var(--radius) - 2px);
  background: var(--popover);
  color: var(--popover-foreground);
  box-shadow: var(--shadow-lg);
}

.sort-menu-caption {
  padding: 2px var(--spacing-sm) 4px;
  color: var(--muted-foreground);
  font-size: var(--font-size-xs);
}

.sort-menu-divider {
  height: 1px;
  margin: 4px 0 6px;
  background: var(--border);
}

.sort-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px var(--spacing-sm);
  border: none;
  border-radius: calc(var(--radius) - 4px);
  background: transparent;
  color: inherit;
  font-size: var(--font-size-sm);
  text-align: left;
  cursor: pointer;
}

.sort-menu-item > span {
  flex: 1;
}

.sort-menu-item:hover {
  background: var(--accent);
}

.sort-menu-item--active {
  color: var(--primary);
  font-weight: 600;
}

.sort-menu-hint {
  opacity: 0.7;
}

.sort-menu-enter-active,
.sort-menu-leave-active {
  transition: opacity 120ms ease, transform 120ms ease;
}
.sort-menu-enter-from,
.sort-menu-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
