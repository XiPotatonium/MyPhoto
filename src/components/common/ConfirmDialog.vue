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
    <div v-if="visible" class="dialog-overlay" @click.self="emit('cancel')">
      <div class="dialog-box">
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
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.dialog-box {
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-xl);
  min-width: 320px;
  max-width: 480px;
  box-shadow: var(--shadow-lg);
}

.dialog-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  margin-bottom: var(--spacing-sm);
}

.dialog-message {
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-xl);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}

.dialog-btn {
  padding: 6px 16px;
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  font-weight: 500;
  transition: all var(--transition-fast);
}

.dialog-btn.cancel {
  border: 1px solid var(--color-border);
  background: var(--color-bg-primary);
}

.dialog-btn.cancel:hover {
  background: var(--color-bg-hover);
}

.dialog-btn.confirm {
  background: var(--color-danger);
  color: white;
}

.dialog-btn.confirm:hover {
  background: var(--color-danger-hover);
}
</style>
