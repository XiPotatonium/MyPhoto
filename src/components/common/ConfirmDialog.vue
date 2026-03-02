<script setup lang="ts">
import { AlertTriangle } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog'
import { cn } from '../../lib/utils'

defineProps<{
  visible: boolean
  title: string
  message: string
  options: { label: string; value: string }[]
}>()

const emit = defineEmits<{
  confirm: [value: string]
  cancel: []
}>()

function handleOpenChange(open: boolean) {
  if (!open) {
    emit('cancel')
  }
}
</script>

<template>
  <Dialog :open="visible" @update:open="handleOpenChange">
    <DialogContent
      :show-close-button="false"
      class="confirm-dialog"
      :class="cn(
        'bg-[hsl(var(--card))] border-[hsl(var(--border))]',
        'dark:bg-[hsl(var(--card))] dark:border-[hsl(var(--border))]'
      )"
    >
      <!-- Header -->
      <DialogHeader class="dialog-header">
        <div class="header-content">
          <div class="header-icon header-icon--destructive">
            <AlertTriangle class="h-5 w-5" />
          </div>
          <div class="header-text">
            <DialogTitle class="dialog-title">
              {{ title }}
            </DialogTitle>
          </div>
        </div>
        <DialogDescription class="dialog-desc">
          {{ message }}
        </DialogDescription>
      </DialogHeader>

      <!-- 底部按钮 -->
      <DialogFooter class="dialog-footer">
        <button
          @click="emit('cancel')"
          class="dialog-btn dialog-btn--secondary"
        >
          取消
        </button>
        <button
          v-for="opt in options"
          :key="opt.value"
          @click="emit('confirm', opt.value)"
          class="dialog-btn dialog-btn--destructive"
        >
          {{ opt.label }}
        </button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<style scoped>
.confirm-dialog {
  max-width: 440px;
  padding: 0;
  overflow: hidden;
  border: 1px solid hsl(var(--border));
  box-shadow: var(--shadow-lg);
}

/* Header 样式 */
.dialog-header {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  padding: var(--spacing-lg) var(--spacing-lg) var(--spacing-md);
}

.header-content {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.header-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.header-icon--destructive {
  background: hsl(var(--destructive));
  color: hsl(var(--destructive-foreground));
}

.header-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dialog-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: hsl(var(--card-foreground));
  letter-spacing: -0.01em;
}

.dialog-desc {
  font-size: var(--font-size-sm);
  color: hsl(var(--muted-foreground));
  line-height: 1.6;
  margin-left: calc(40px + var(--spacing-md));
}

/* 底部按钮样式 */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding: var(--spacing-md) var(--spacing-lg) var(--spacing-lg);
  gap: var(--spacing-sm);
}

.dialog-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 36px;
  padding: 0 20px;
  font-size: var(--font-size-sm);
  font-weight: 500;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  min-width: 88px;
}

.dialog-btn--secondary {
  color: hsl(var(--foreground));
  background: hsl(var(--background));
  border: 1px solid hsl(var(--border));
}

.dialog-btn--secondary:hover {
  background: hsl(var(--accent));
  color: hsl(var(--accent-foreground));
}

.dialog-btn--destructive {
  color: hsl(var(--destructive-foreground));
  background: hsl(var(--destructive));
  min-width: 120px;
}

.dialog-btn--destructive:hover {
  background: hsl(var(--destructive) / 0.9);
}
</style>
