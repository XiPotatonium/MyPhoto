<script setup lang="ts">
import { ref, computed } from 'vue'
import { MapPin, Loader2, Compass, Globe, Navigation } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog'
import { Button } from '../ui/button'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { cn } from '../../lib/utils'

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
  if (latitude.value === '') return false
  const lat = parseFloat(latitude.value)
  return !isNaN(lat) && lat >= -90 && lat <= 90
})

const isValidLongitude = computed(() => {
  if (longitude.value === '') return false
  const lon = parseFloat(longitude.value)
  return !isNaN(lon) && lon >= -180 && lon <= 180
})

const canConfirm = computed(() => {
  return isValidLatitude.value && isValidLongitude.value && !loading.value
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

function reset() {
  latitude.value = ''
  longitude.value = ''
  loading.value = false
}

defineExpose({ loading, reset })
</script>

<template>
  <Dialog :open="visible" @update:open="(open) => !open && handleCancel()">
    <DialogContent
      :show-close-button="false"
      class="sm:max-w-[440px] p-0 overflow-hidden border shadow-xl"
      :class="cn(
        'bg-[hsl(var(--card))] border-[hsl(var(--border))]',
        'dark:bg-[hsl(var(--card))] dark:border-[hsl(var(--border))]'
      )"
    >
      <!-- Header -->
      <DialogHeader class="dialog-header">
        <div class="header-content">
          <div class="header-icon">
            <MapPin class="h-5 w-5" />
          </div>
          <div class="header-text">
            <DialogTitle class="dialog-title">
              添加 GPS 信息
            </DialogTitle>
            <DialogDescription class="dialog-desc">
              为选中的图像添加地理位置坐标
            </DialogDescription>
          </div>
        </div>
      </DialogHeader>

      <!-- 表单内容 -->
      <div class="form-content">
        <!-- 纬度输入 -->
        <div class="input-group">
          <div class="input-label">
            <Compass class="input-icon" />
            <Label for="latitude" class="label-text">
              纬度 Latitude
            </Label>
          </div>
          <div class="input-wrapper">
            <Input
              id="latitude"
              v-model="latitude"
              type="text"
              placeholder="例如: 39.9042"
              :disabled="loading"
              class="gps-input"
              :class="cn(
                latitude !== '' && !isValidLatitude && 'input-error'
              )"
            />
            <div class="input-suffix">
              °N
            </div>
          </div>
          <p v-if="latitude !== '' && !isValidLatitude" class="error-msg">
            <span class="error-dot" />
            纬度必须在 -90 到 90 之间
          </p>
        </div>

        <!-- 经度输入 -->
        <div class="input-group">
          <div class="input-label">
            <Navigation class="input-icon" />
            <Label for="longitude" class="label-text">
              经度 Longitude
            </Label>
          </div>
          <div class="input-wrapper">
            <Input
              id="longitude"
              v-model="longitude"
              type="text"
              placeholder="例如: 116.4074"
              :disabled="loading"
              class="gps-input"
              :class="cn(
                longitude !== '' && !isValidLongitude && 'input-error'
              )"
            />
            <div class="input-suffix">
              °E
            </div>
          </div>
          <p v-if="longitude !== '' && !isValidLongitude" class="error-msg">
            <span class="error-dot" />
            经度必须在 -180 到 180 之间
          </p>
        </div>

        <!-- 提示信息 -->
        <div class="info-box">
          <div class="info-content">
            <Globe class="info-icon" />
            <div class="info-text">
              <p class="info-title">查找坐标</p>
              <p class="info-desc">
                访问
                <a
                  href="https://jingweidu.bmcx.com"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="info-link"
                >
                  经纬度查询网站
                </a>
                获取精确的地理位置坐标
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部按钮 -->
      <DialogFooter class="dialog-footer">
        <button
          :disabled="loading"
          @click="handleCancel"
          class="dialog-btn dialog-btn--secondary"
        >
          取消
        </button>
        <button
          :disabled="!canConfirm"
          @click="handleConfirm"
          :class="cn('dialog-btn dialog-btn--primary', !canConfirm && 'dialog-btn--disabled')"
        >
          <Loader2 v-if="loading" class="mr-2 h-4 w-4 animate-spin" />
          <MapPin v-else class="mr-2 h-4 w-4" />
          <span>{{ loading ? '保存中...' : '确认添加' }}</span>
        </button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<style scoped>
/* Header 样式 */
.dialog-header {
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
  background: hsl(var(--primary));
  color: hsl(var(--primary-foreground));
  flex-shrink: 0;
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
}

/* 表单内容样式 */
.form-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  padding: 0 var(--spacing-lg) var(--spacing-lg);
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.input-label {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.input-icon {
  width: 16px;
  height: 16px;
  color: hsl(var(--primary));
  flex-shrink: 0;
}

.label-text {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: hsl(var(--foreground));
}

.input-wrapper {
  position: relative;
}

.gps-input {
  height: 40px;
  padding-right: 44px;
  background: hsl(var(--background));
  border-color: hsl(var(--input));
  color: hsl(var(--foreground));
  font-size: var(--font-size-sm);
}

.gps-input::placeholder {
  color: hsl(var(--muted-foreground));
}

.input-suffix {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: var(--font-size-sm);
  color: hsl(var(--muted-foreground));
  font-weight: 500;
  pointer-events: none;
}

.input-error {
  border-color: hsl(var(--destructive));
}

.input-error:focus-visible {
  ring-color: hsl(var(--destructive));
}

.error-msg {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-xs);
  color: hsl(var(--destructive));
}

.error-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: hsl(var(--destructive));
  flex-shrink: 0;
}

/* 提示信息样式 */
.info-box {
  border-radius: var(--radius-md);
  background: hsl(var(--muted) / 0.6);
  border: 1px solid hsl(var(--border));
  padding: var(--spacing-md);
}

.info-content {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-sm);
}

.info-icon {
  width: 16px;
  height: 16px;
  color: hsl(var(--muted-foreground));
  margin-top: 2px;
  flex-shrink: 0;
}

.info-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-title {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: hsl(var(--foreground));
}

.info-desc {
  font-size: var(--font-size-xs);
  color: hsl(var(--muted-foreground));
  line-height: 1.5;
}

.info-link {
  color: hsl(var(--primary));
  text-decoration: underline;
  text-underline-offset: 2px;
}

.info-link:hover {
  color: hsl(var(--primary) / 0.8);
}

/* 底部按钮样式 */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding: var(--spacing-md) var(--spacing-lg);
  border-top: 1px solid hsl(var(--border));
  background: hsl(var(--muted) / 0.3);
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

.dialog-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.dialog-btn--secondary {
  color: hsl(var(--foreground));
  background: hsl(var(--background));
  border: 1px solid hsl(var(--border));
}

.dialog-btn--secondary:hover:not(:disabled) {
  background: hsl(var(--accent));
  color: hsl(var(--accent-foreground));
}

.dialog-btn--primary {
  color: hsl(var(--primary-foreground));
  background: hsl(var(--primary));
  min-width: 120px;
}

.dialog-btn--primary:hover:not(:disabled) {
  background: hsl(var(--primary) / 0.9);
}

.dialog-btn--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
