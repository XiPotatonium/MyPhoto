<script setup lang="ts">
import { ref, computed } from 'vue'
import { FileEdit, Loader2, Compass, Navigation, Camera, Aperture, Clock, Gauge, Ruler, Calendar } from 'lucide-vue-next'
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
import type { ExifInfo } from '../../types/exif'

export interface ExifWriteRequest {
  datetime?: string | null
  cameraModel?: string | null
  lensModel?: string | null
  focalLength?: number | null
  shutterSpeed?: string | null
  aperture?: number | null
  iso?: number | null
  gpsLatitude?: number | null
  gpsLongitude?: number | null
}

defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  confirm: [fields: ExifWriteRequest]
  cancel: []
}>()

const loading = ref(false)

// Form fields
const datetime = ref('')
const cameraModel = ref('')
const lensModel = ref('')
const focalLength = ref('')
const shutterSpeed = ref('')
const aperture = ref('')
const iso = ref('')
const latitude = ref('')
const longitude = ref('')

// ── Validation ──────────────────────────────────────────────────────────────

const isValidLatitude = computed(() => {
  if (latitude.value === '') return true
  const v = parseFloat(latitude.value)
  return !isNaN(v) && v >= -90 && v <= 90
})

const isValidLongitude = computed(() => {
  if (longitude.value === '') return true
  const v = parseFloat(longitude.value)
  return !isNaN(v) && v >= -180 && v <= 180
})

const isValidShutterSpeed = computed(() => {
  if (shutterSpeed.value === '') return true
  const s = shutterSpeed.value.trim()
  if (s.includes('/')) {
    const parts = s.split('/')
    if (parts.length !== 2) return false
    const num = Number(parts[0].trim())
    const denom = Number(parts[1].trim())
    return Number.isInteger(num) && Number.isInteger(denom) && num > 0 && denom > 0
  }
  const v = parseFloat(s)
  return !isNaN(v) && v > 0
})

const isValidFocalLength = computed(() => {
  if (focalLength.value === '') return true
  const v = parseFloat(focalLength.value)
  return !isNaN(v) && v > 0
})

const isValidAperture = computed(() => {
  if (aperture.value === '') return true
  const v = parseFloat(aperture.value)
  return !isNaN(v) && v > 0
})

const isValidIso = computed(() => {
  if (iso.value === '') return true
  const v = parseInt(iso.value, 10)
  return !isNaN(v) && v > 0 && v <= 65535
})

const isValidDatetime = computed(() => {
  if (datetime.value === '') return true
  // Accept YYYY:MM:DD HH:MM:SS or YYYY-MM-DD HH:MM:SS
  return /^\d{4}[:\-]\d{2}[:\-]\d{2} \d{2}:\d{2}:\d{2}$/.test(datetime.value.trim())
})

const hasAnyField = computed(() => {
  return [datetime, cameraModel, lensModel, focalLength, shutterSpeed, aperture, iso, latitude, longitude]
    .some(f => f.value.trim() !== '')
})

const allValid = computed(() => {
  return isValidLatitude.value && isValidLongitude.value
    && isValidShutterSpeed.value && isValidFocalLength.value
    && isValidAperture.value && isValidIso.value && isValidDatetime.value
})

const canConfirm = computed(() => hasAnyField.value && allValid.value && !loading.value)

// ── Actions ──────────────────────────────────────────────────────────────────

function handleConfirm() {
  if (!canConfirm.value) return

  // Normalize datetime separator to colon style (YYYY:MM:DD HH:MM:SS)
  let dt: string | undefined = undefined
  if (datetime.value.trim()) {
    dt = datetime.value.trim().replace(/^(\d{4})-(\d{2})-(\d{2})/, '$1:$2:$3')
  }

  const fields: ExifWriteRequest = {
    datetime: dt || null,
    cameraModel: cameraModel.value.trim() || null,
    lensModel: lensModel.value.trim() || null,
    focalLength: focalLength.value.trim() ? parseFloat(focalLength.value) : null,
    shutterSpeed: shutterSpeed.value.trim() || null,
    aperture: aperture.value.trim() ? parseFloat(aperture.value) : null,
    iso: iso.value.trim() ? parseInt(iso.value, 10) : null,
    gpsLatitude: latitude.value.trim() ? parseFloat(latitude.value) : null,
    gpsLongitude: longitude.value.trim() ? parseFloat(longitude.value) : null,
  }

  emit('confirm', fields)
}

function handleCancel() {
  if (loading.value) return
  emit('cancel')
}

function reset() {
  datetime.value = ''
  cameraModel.value = ''
  lensModel.value = ''
  focalLength.value = ''
  shutterSpeed.value = ''
  aperture.value = ''
  iso.value = ''
  latitude.value = ''
  longitude.value = ''
  loading.value = false
}

/** Pre-fill the form from existing ExifInfo (for future use). */
function initWithExif(_exifInfo: ExifInfo) {
  // Reserved for future pre-fill support
}

defineExpose({ loading, reset, initWithExif })
</script>

<template>
  <Dialog :open="visible" @update:open="(open) => !open && handleCancel()">
    <DialogContent
      :show-close-button="false"
      class="sm:max-w-[680px] p-0 overflow-hidden"
    >
      <!-- Header -->
      <DialogHeader class="px-6 pt-5 pb-4">
        <div class="flex items-center gap-3">
          <div class="flex items-center justify-center w-10 h-10 rounded-md bg-primary text-primary-foreground shrink-0">
            <FileEdit class="h-5 w-5" />
          </div>
          <div class="flex flex-col gap-0.5">
            <DialogTitle>修改 EXIF 信息</DialogTitle>
            <DialogDescription>编辑选中图像的 EXIF 元数据，留空的字段将保持原值不变</DialogDescription>
          </div>
        </div>
      </DialogHeader>

      <!-- Form body -->
      <div class="px-6 pb-5 grid grid-cols-2 gap-x-4 gap-y-4">

        <!-- 拍摄时间 -->
        <Field :data-invalid="datetime !== '' && !isValidDatetime ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <Calendar class="h-3.5 w-3.5 text-primary" />
            拍摄时间
          </FieldLabel>
          <Input
            v-model="datetime"
            type="text"
            placeholder="2024:06:01 12:00:00"
            :disabled="loading"
            :class="datetime !== '' && !isValidDatetime ? 'border-destructive focus-visible:ring-destructive' : ''"
          />
          <FieldError v-if="datetime !== '' && !isValidDatetime">
            格式应为 YYYY:MM:DD HH:MM:SS
          </FieldError>
        </Field>

        <!-- 相机型号 -->
        <Field>
          <FieldLabel class="flex items-center gap-1.5">
            <Camera class="h-3.5 w-3.5 text-primary" />
            相机型号
          </FieldLabel>
          <Input
            v-model="cameraModel"
            type="text"
            placeholder="例如: FUJIFILM X-T5"
            :disabled="loading"
          />
        </Field>

        <!-- 镜头型号（跨两列） -->
        <Field class="col-span-2">
          <FieldLabel class="flex items-center gap-1.5">
            <Ruler class="h-3.5 w-3.5 text-primary" />
            镜头型号
          </FieldLabel>
          <Input
            v-model="lensModel"
            type="text"
            placeholder="例如: XF 16-55mm F2.8 R LM WR"
            :disabled="loading"
          />
        </Field>

        <!-- 焦距 -->
        <Field :data-invalid="focalLength !== '' && !isValidFocalLength ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <Gauge class="h-3.5 w-3.5 text-primary" />
            焦距 (mm)
          </FieldLabel>
          <Input
            v-model="focalLength"
            type="text"
            placeholder="例如: 35"
            :disabled="loading"
            :class="focalLength !== '' && !isValidFocalLength ? 'border-destructive focus-visible:ring-destructive' : ''"
          />
          <FieldError v-if="focalLength !== '' && !isValidFocalLength">
            请输入有效的正数
          </FieldError>
        </Field>

        <!-- 光圈值 -->
        <Field :data-invalid="aperture !== '' && !isValidAperture ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <Aperture class="h-3.5 w-3.5 text-primary" />
            光圈 (f/)
          </FieldLabel>
          <Input
            v-model="aperture"
            type="text"
            placeholder="例如: 2.8"
            :disabled="loading"
            :class="aperture !== '' && !isValidAperture ? 'border-destructive focus-visible:ring-destructive' : ''"
          />
          <FieldError v-if="aperture !== '' && !isValidAperture">
            请输入有效的正数
          </FieldError>
        </Field>

        <!-- 快门速度 -->
        <Field :data-invalid="shutterSpeed !== '' && !isValidShutterSpeed ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <Clock class="h-3.5 w-3.5 text-primary" />
            快门速度
          </FieldLabel>
          <Input
            v-model="shutterSpeed"
            type="text"
            placeholder="例如: 1/500 或 2"
            :disabled="loading"
            :class="shutterSpeed !== '' && !isValidShutterSpeed ? 'border-destructive focus-visible:ring-destructive' : ''"
          />
          <FieldError v-if="shutterSpeed !== '' && !isValidShutterSpeed">
            格式应为 1/500 或 2（正数）
          </FieldError>
        </Field>

        <!-- ISO -->
        <Field :data-invalid="iso !== '' && !isValidIso ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <span class="text-primary font-bold text-xs">ISO</span>
            感光度
          </FieldLabel>
          <Input
            v-model="iso"
            type="text"
            placeholder="例如: 400"
            :disabled="loading"
            :class="iso !== '' && !isValidIso ? 'border-destructive focus-visible:ring-destructive' : ''"
          />
          <FieldError v-if="iso !== '' && !isValidIso">
            请输入 1–65535 之间的整数
          </FieldError>
        </Field>

        <!-- 分隔线 -->
        <div class="col-span-2 border-t border-border pt-1">
          <p class="text-xs text-muted-foreground font-medium">GPS 坐标（可选）</p>
        </div>

        <!-- 纬度 -->
        <Field :data-invalid="latitude !== '' && !isValidLatitude ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <Compass class="h-3.5 w-3.5 text-primary" />
            纬度 Latitude
          </FieldLabel>
          <div class="relative">
            <Input
              v-model="latitude"
              type="text"
              placeholder="例如: 39.9042"
              :disabled="loading"
              class="pr-9"
              :class="latitude !== '' && !isValidLatitude ? 'border-destructive focus-visible:ring-destructive' : ''"
            />
            <span class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-muted-foreground font-medium pointer-events-none">°N</span>
          </div>
          <FieldError v-if="latitude !== '' && !isValidLatitude">
            纬度必须在 -90 到 90 之间
          </FieldError>
        </Field>

        <!-- 经度 -->
        <Field :data-invalid="longitude !== '' && !isValidLongitude ? true : undefined">
          <FieldLabel class="flex items-center gap-1.5">
            <Navigation class="h-3.5 w-3.5 text-primary" />
            经度 Longitude
          </FieldLabel>
          <div class="relative">
            <Input
              v-model="longitude"
              type="text"
              placeholder="例如: 116.4074"
              :disabled="loading"
              class="pr-9"
              :class="longitude !== '' && !isValidLongitude ? 'border-destructive focus-visible:ring-destructive' : ''"
            />
            <span class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-muted-foreground font-medium pointer-events-none">°E</span>
          </div>
          <FieldError v-if="longitude !== '' && !isValidLongitude">
            经度必须在 -180 到 180 之间
          </FieldError>
        </Field>

      </div>

      <!-- Footer -->
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
          <FileEdit v-else class="mr-2 h-4 w-4" />
          {{ loading ? '保存中...' : '确认修改' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
