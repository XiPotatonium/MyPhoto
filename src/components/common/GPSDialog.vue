<script setup lang="ts">
import { ref, computed } from 'vue'

defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  confirm: [latitude: number, longitude: number]
  cancel: []
}>()

const latitude = ref('')
const longitude = ref('')
const loading = ref(false)

// 验证经纬度格式
const isValidLatitude = computed(() => {
  if (latitude.value === '') return false // 空值视为无效
  const lat = parseFloat(latitude.value)
  return !isNaN(lat) && lat >= -90 && lat <= 90
})

const isValidLongitude = computed(() => {
  if (longitude.value === '') return false // 空值视为无效
  const lon = parseFloat(longitude.value)
  return !isNaN(lon) && lon >= -180 && lon <= 180
})

const canConfirm = computed(() => {
  return (
    isValidLatitude.value &&
    isValidLongitude.value &&
    !loading.value
  )
})

function handleConfirm() {
  if (!canConfirm.value) return
  const lat = parseFloat(latitude.value)
  const lon = parseFloat(longitude.value)
  emit('confirm', lat, lon)
}

function handleCancel() {
  if (loading.value) return
  emit('cancel')
}

// 重置表单
function reset() {
  latitude.value = ''
  longitude.value = ''
  loading.value = false
}

defineExpose({ loading, reset })
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-overlay" @click.self="handleCancel">
      <div class="dialog-box">
        <div class="dialog-title">添加GPS信息</div>
        
        <div class="dialog-content">
          <div class="form-group">
            <label for="latitude">纬度 (Latitude)</label>
            <input
              id="latitude"
              v-model="latitude"
              type="text"
              placeholder="例如: 39.9042"
              :disabled="loading"
              :class="{ invalid: latitude !== '' && !isValidLatitude }"
            />
            <div v-if="latitude !== '' && !isValidLatitude" class="error-msg">
              纬度必须在 -90 到 90 之间
            </div>
          </div>

          <div class="form-group">
            <label for="longitude">经度 (Longitude)</label>
            <input
              id="longitude"
              v-model="longitude"
              type="text"
              placeholder="例如: 116.4074"
              :disabled="loading"
              :class="{ invalid: longitude !== '' && !isValidLongitude }"
            />
            <div v-if="longitude !== '' && !isValidLongitude" class="error-msg">
              经度必须在 -180 到 180 之间
            </div>
          </div>

          <div class="hint">
            <span class="hint-icon">ℹ️</span>
            <span>
              可以在
              <a href="https://jingweidu.bmcx.com" target="_blank" rel="noopener noreferrer">
                https://jingweidu.bmcx.com
              </a>
              上查找具体位置的经纬度坐标
            </span>
          </div>
        </div>

        <div class="dialog-actions">
          <button class="dialog-btn cancel" @click="handleCancel" :disabled="loading">
            取消
          </button>
          <button
            class="dialog-btn confirm"
            @click="handleConfirm"
            :disabled="!canConfirm"
          >
            <span v-if="loading" class="loading-spinner"></span>
            <span v-else>确认</span>
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
  min-width: 420px;
  max-width: 520px;
  box-shadow: var(--shadow-lg);
}

.dialog-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  margin-bottom: var(--spacing-lg);
}

.dialog-content {
  margin-bottom: var(--spacing-xl);
}

.form-group {
  margin-bottom: var(--spacing-md);
}

.form-group label {
  display: block;
  font-size: var(--font-size-sm);
  font-weight: 500;
  margin-bottom: var(--spacing-xs);
  color: var(--color-text-primary);
}

.form-group input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-base);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  transition: border-color var(--transition-fast);
}

.form-group input:focus {
  outline: none;
  border-color: var(--color-accent);
}

.form-group input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-group input.invalid {
  border-color: var(--color-danger);
}

.error-msg {
  margin-top: var(--spacing-xs);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
}

.hint {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm);
  background: var(--color-bg-secondary);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.hint-icon {
  flex-shrink: 0;
  font-size: 14px;
}

.hint a {
  color: var(--color-accent);
  text-decoration: none;
}

.hint a:hover {
  text-decoration: underline;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}

.dialog-btn {
  padding: 8px 20px;
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  font-weight: 500;
  transition: all var(--transition-fast);
  min-width: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.dialog-btn.cancel {
  border: 1px solid var(--color-border);
  background: var(--color-bg-primary);
}

.dialog-btn.cancel:hover:not(:disabled) {
  background: var(--color-bg-hover);
}

.dialog-btn.confirm {
  background: var(--color-accent);
  color: white;
  border: none;
}

.dialog-btn.confirm:hover:not(:disabled) {
  opacity: 0.9;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
