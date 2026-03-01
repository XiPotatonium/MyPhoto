<script setup lang="ts">
import { ref, computed } from 'vue'
import { MapPin, Loader2, Info } from 'lucide-vue-next'
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
  <Dialog :open="visible" @update:open="(open) => !open && handleCancel()">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <MapPin class="h-5 w-5" />
          添加GPS信息
        </DialogTitle>
        <DialogDescription>
          为选中的图像添加地理位置信息
        </DialogDescription>
      </DialogHeader>
      
      <div class="grid gap-4 py-4">
        <div class="grid gap-2">
          <Label for="latitude">纬度 (Latitude)</Label>
          <Input
            id="latitude"
            v-model="latitude"
            type="text"
            placeholder="例如: 39.9042"
            :disabled="loading"
            :class="cn(
              latitude !== '' && !isValidLatitude && 'border-destructive focus-visible:ring-destructive'
            )"
          />
          <p v-if="latitude !== '' && !isValidLatitude" class="text-sm text-destructive">
            纬度必须在 -90 到 90 之间
          </p>
        </div>

        <div class="grid gap-2">
          <Label for="longitude">经度 (Longitude)</Label>
          <Input
            id="longitude"
            v-model="longitude"
            type="text"
            placeholder="例如: 116.4074"
            :disabled="loading"
            :class="cn(
              longitude !== '' && !isValidLongitude && 'border-destructive focus-visible:ring-destructive'
            )"
          />
          <p v-if="longitude !== '' && !isValidLongitude" class="text-sm text-destructive">
            经度必须在 -180 到 180 之间
          </p>
        </div>

        <div class="flex items-start gap-2 rounded-md bg-muted p-3 text-sm text-muted-foreground">
          <Info class="h-4 w-4 shrink-0 mt-0.5" />
          <span>
            可以在
            <a 
              href="https://jingweidu.bmcx.com" 
              target="_blank" 
              rel="noopener noreferrer"
              class="text-primary hover:underline"
            >
              https://jingweidu.bmcx.com
            </a>
            上查找具体位置的经纬度坐标
          </span>
        </div>
      </div>

      <DialogFooter>
        <Button variant="outline" :disabled="loading" @click="handleCancel">
          取消
        </Button>
        <Button :disabled="!canConfirm" @click="handleConfirm">
          <Loader2 v-if="loading" class="mr-2 h-4 w-4 animate-spin" />
          <span v-else>确认</span>
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
