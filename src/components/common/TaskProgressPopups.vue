<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { AlertTriangle, Check, Loader2 } from 'lucide-vue-next'
import { useAppState } from '../../stores/appState'
import type { TaskProgress, TaskProgressPayload } from '../../types/task'

// 0.3.4：左上角任务进度浮层。
// 后端批量任务（HDR 转换 / EXIF 写入）每完成一项推送 task-progress 事件，
// 这里统一更新 appState.tasks；全部完成后停留约 1.8s 自动收起。
const appState = useAppState()
const timers = new Map<string, number>()
let unlisten: UnlistenFn | null = null
let disposed = false

function progressPercent(task: TaskProgress): number {
  if (task.total <= 0) return task.done ? 100 : 0
  return Math.round((task.processed / task.total) * 100)
}

watch(
  () => appState.state.tasks,
  (tasks) => {
    for (const task of tasks) {
      if (task.done && !timers.has(task.id)) {
        const timer = window.setTimeout(() => {
          timers.delete(task.id)
          appState.removeTask(task.id)
        }, 1800)
        timers.set(task.id, timer)
      }
    }
    // 任务被外部移除（例如手动清理）时同步清掉定时器
    for (const [id, timer] of timers) {
      if (!tasks.some(task => task.id === id)) {
        window.clearTimeout(timer)
        timers.delete(id)
      }
    }
  },
  { deep: true, immediate: true },
)

onMounted(async () => {
  try {
    const stop = await listen<TaskProgressPayload>('task-progress', (event) => {
      const payload = event.payload
      appState.updateTask(payload.taskId, {
        processed: payload.processed,
        total: payload.total,
        current: payload.current,
        failed: payload.failed,
      })
    })
    if (disposed) {
      stop()
    } else {
      unlisten = stop
    }
  } catch (e) {
    // 非 Tauri 环境（例如直接用浏览器打开 Vite 页面）没有事件系统，忽略即可
    console.warn('task-progress listener unavailable:', e)
  }
})

onUnmounted(() => {
  disposed = true
  unlisten?.()
  unlisten = null
  for (const timer of timers.values()) window.clearTimeout(timer)
  timers.clear()
})
</script>

<template>
  <div class="task-stack" aria-live="polite">
    <TransitionGroup name="task">
      <div
        v-for="task in appState.state.tasks"
        :key="task.id"
        class="task-card"
        :class="{
          'task-card--done': task.done,
          'task-card--failed': task.done && task.failed > 0,
        }"
      >
        <div class="task-head">
          <span class="task-icon">
            <Check v-if="task.done && task.failed === 0" class="h-4 w-4" />
            <AlertTriangle v-else-if="task.done" class="h-4 w-4" />
            <Loader2 v-else class="h-4 w-4 animate-spin" />
          </span>
          <span class="task-title">{{ task.title }}</span>
          <span class="task-count">{{ task.processed }}/{{ task.total }}</span>
        </div>

        <div class="task-file" :title="task.current ?? ''">
          <template v-if="task.done">
            {{ task.failed > 0 ? `完成，${task.failed} 张失败` : '全部完成' }}
          </template>
          <template v-else>
            {{ task.current ? `正在处理 ${task.current}` : '准备中…' }}
          </template>
        </div>

        <div class="task-bar">
          <div
            class="task-bar-fill"
            :class="{ 'task-bar-fill--done': task.done, 'task-bar-fill--failed': task.done && task.failed > 0 }"
            :style="{ width: progressPercent(task) + '%' }"
          />
        </div>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.task-stack {
  position: absolute;
  top: 98px;
  left: 12px;
  z-index: var(--z-toast);
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 320px;
  pointer-events: none;
}

.task-card {
  pointer-events: auto;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--popover);
  color: var(--popover-foreground);
  box-shadow: 0 6px 16px -6px rgb(0 0 0 / 0.22),
              0 2px 6px -2px rgb(0 0 0 / 0.10);
}

:global(html.dark) .task-card {
  box-shadow: 0 8px 20px -6px rgb(0 0 0 / 0.60),
              0 2px 8px -2px rgb(0 0 0 / 0.40);
}

.task-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.task-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  border-radius: 50%;
  background: var(--primary);
  color: var(--primary-foreground);
}

.task-card--done .task-icon {
  background: var(--primary);
}

.task-card--failed .task-icon {
  background: var(--destructive);
  color: #fff;
}

.task-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
}

.task-count {
  flex-shrink: 0;
  font-size: var(--font-size-xs);
  font-variant-numeric: tabular-nums;
  color: var(--muted-foreground);
}

.task-file {
  margin-top: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-size-xs);
  color: var(--muted-foreground);
}

.task-card--failed .task-file {
  color: var(--destructive);
}

.task-bar {
  margin-top: 8px;
  height: 4px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--muted);
}

.task-bar-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--primary);
  transition: width 220ms cubic-bezier(0.4, 0, 0.2, 1);
}

.task-bar-fill--done {
  background: var(--primary);
}

.task-bar-fill--failed {
  background: var(--destructive);
}

/* 浮层进出场 */
.task-enter-active,
.task-leave-active {
  transition: opacity 180ms ease, transform 180ms ease;
}

.task-enter-from,
.task-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
