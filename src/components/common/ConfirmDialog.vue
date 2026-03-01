<script setup lang="ts">
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
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
      <div v-if="visible" class="dialog-overlay" @click.self="emit('cancel')">
        <div class="dialog-box">
          <div class="dialog-icon">⚠️</div>
          <div class="dialog-title">{{ title }}</div>
          <div class="dialog-message">{{ message }}</div>
          <div class="dialog-actions">
            <button class="dialog-btn cancel" @click="emit('cancel')">取消</button>
            <button
              v-for="opt in options"
              :key="opt.value"
              class="dialog-btn confirm"
              @click="emit('confirm', opt.value)"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-scrim);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: var(--spacing-4);
}

.dialog-box {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  padding: var(--spacing-6);
  min-width: 360px;
  max-width: 520px;
  box-shadow: var(--shadow-xl);
  text-align: center;
}

.dialog-icon {
  font-size: 48px;
  margin-bottom: var(--spacing-3);
}

.dialog-title {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-2);
}

.dialog-message {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: var(--line-height-relaxed);
  margin-bottom: var(--spacing-6);
}

.dialog-actions {
  display: flex;
  justify-content: center;
  gap: var(--spacing-3);
  flex-wrap: wrap;
}

.dialog-btn {
  padding: var(--spacing-2) var(--spacing-5);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  transition: all var(--transition-fast);
  min-width: 80px;
}

.dialog-btn.cancel {
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-secondary);
}

.dialog-btn.cancel:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-border-strong);
  color: var(--color-text-primary);
}

.dialog-btn.confirm {
  background: var(--color-danger);
  color: white;
  border: 1px solid var(--color-danger);
  box-shadow: 0 1px 2px rgba(220, 38, 38, 0.2);
}

.dialog-btn.confirm:hover {
  background: var(--color-danger-hover);
  border-color: var(--color-danger-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(220, 38, 38, 0.25);
}

.dialog-btn:active {
  transform: scale(0.98);
}

/* Dialog animation */
.dialog-enter-active,
.dialog-leave-active {
  transition: all var(--transition-normal);
}

.dialog-enter-from,
.dialog-leave-to {
  opacity: 0;
}

.dialog-enter-from .dialog-box,
.dialog-leave-to .dialog-box {
  opacity: 0;
  transform: scale(0.95) translateY(10px);
}

.dialog-enter-active .dialog-box,
.dialog-leave-active .dialog-box {
  transition: all var(--transition-spring);
}
</style>
