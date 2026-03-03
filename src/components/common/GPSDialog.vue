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
import { Input } from '../ui/input'
import { Button } from '../ui/button'
import {
  Field,
  FieldLabel,
  FieldError,
} from '../ui/field'

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
  if (latitude.value === '') return true
  const lat = parseFloat(latitude.value)
  return !isNaN(lat) && lat >= -90 && lat <= 90
})

const isValidLongitude = computed(() => {
  if (longitude.value === '') return true
  const lon = parseFloat(longitude.value)
  return !isNaN(lon) && lon >= -180 && lon <= 180
})

const canConfirm = computed(() => {
  return latitude.value !== '' && longitude.value !== ''
    && isValidLatitude.value && isValidLongitude.value && !loading.value
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
      class="sm:max-w-[440px] p-0 overflow-hidden"
    >
      <!-- Header -->
      <DialogHeader class="px-6 pt-6 pb-4">
        <div class="flex items-center gap-3">
          <div class="flex items-center justify-center w-10 h-10 rounded-md bg-primary text-primary-foreground shrink-0">
            <MapPin class="h-5 w-5" />
          </div>
          <div class="flex flex-col gap-0.5">
            <DialogTitle>添加 GPS 信息</DialogTitle>
            <DialogDescription>为选中的图像添加地理位置坐标</DialogDescription>
          </div>
        </div>
      </DialogHeader>

      <!-- 表单内容 -->
      <div class="flex flex-col gap-5 px-6 pb-6">
        <!-- 纬度输入 -->
        <Field :data-invalid="latitude !== '' && !isValidLatitude ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <Compass class="h-4 w-4 text-primary" />
            纬度 Latitude
          </FieldLabel>
          <div class="relative">
            <Input
              id="latitude"
              v-model="latitude"
              type="text"
              placeholder="例如: 39.9042"
              :disabled="loading"
              class="pr-11"
              :class="latitude !== '' && !isValidLatitude ? 'border-destructive focus-visible:ring-destructive' : ''"
            />
            <span class="absolute right-3 top-1/2 -translate-y-1/2 text-sm text-muted-foreground font-medium pointer-events-none">°N</span>
          </div>
          <FieldError v-if="latitude !== '' && !isValidLatitude">
            纬度必须在 -90 到 90 之间
          </FieldError>
        </Field>

        <!-- 经度输入 -->
        <Field :data-invalid="longitude !== '' && !isValidLongitude ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <Navigation class="h-4 w-4 text-primary" />
            经度 Longitude
          </FieldLabel>
          <div class="relative">
            <Input
              id="longitude"
              v-model="longitude"
              type="text"
              placeholder="例如: 116.4074"
              :disabled="loading"
              class="pr-11"
              :class="longitude !== '' && !isValidLongitude ? 'border-destructive focus-visible:ring-destructive' : ''"
            />
            <span class="absolute right-3 top-1/2 -translate-y-1/2 text-sm text-muted-foreground font-medium pointer-events-none">°E</span>
          </div>
          <FieldError v-if="longitude !== '' && !isValidLongitude">
            经度必须在 -180 到 180 之间
          </FieldError>
        </Field>

        <!-- 提示信息 -->
        <div class="rounded-md bg-muted/60 border border-border p-4">
          <div class="flex items-start gap-2">
            <Globe class="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
            <div class="flex flex-col gap-1">
              <p class="text-sm font-medium">查找坐标</p>
              <p class="text-xs text-muted-foreground leading-relaxed">
                访问
                <a
                  href="https://jingweidu.bmcx.com"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-primary underline underline-offset-2 hover:opacity-80"
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
      <DialogFooter class="px-6 py-4 border-t border-border bg-muted/30">
        <Button
          variant="outline"
          :disabled="loading"
          @click="handleCancel"
        >
          取消
        </Button>
        <Button
          :disabled="!canConfirm"
          @click="handleConfirm"
        >
          <Loader2 v-if="loading" class="mr-2 h-4 w-4 animate-spin" />
          <MapPin v-else class="mr-2 h-4 w-4" />
          {{ loading ? '保存中...' : '确认添加' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
