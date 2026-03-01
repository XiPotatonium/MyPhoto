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
      class="sm:max-w-[480px] border p-0 overflow-hidden shadow-2xl"
      :class="cn(
        'bg-[hsl(var(--card))] border-[hsl(var(--border))]',
        'dark:bg-[hsl(var(--card))] dark:border-[hsl(var(--border))]'
      )"
    >
      <!-- 顶部装饰区域 -->
      <div class="relative overflow-hidden">
        <!-- Light mode 渐变 -->
        <div class="absolute inset-0 bg-gradient-to-br from-[hsl(var(--primary))]/10 via-transparent to-[hsl(var(--accent))]/10 dark:hidden" />
        <!-- Dark mode 渐变 -->
        <div class="absolute inset-0 bg-gradient-to-br from-[hsl(var(--primary))]/20 via-transparent to-[hsl(var(--accent))]/15 hidden dark:block" />
        
        <!-- 装饰光晕 -->
        <div class="absolute -top-16 -right-16 w-32 h-32 bg-[hsl(var(--primary))]/20 rounded-full blur-3xl dark:bg-[hsl(var(--primary))]/15" />
        <div class="absolute -bottom-8 -left-8 w-24 h-24 bg-[hsl(var(--accent))]/15 rounded-full blur-2xl dark:bg-[hsl(var(--accent))]/10" />
        
        <DialogHeader class="relative z-10 px-8 pt-8 pb-6">
          <div class="flex items-center gap-4 mb-3">
            <div class="flex items-center justify-center w-12 h-12 rounded-2xl bg-[hsl(var(--primary))] shadow-lg shadow-[hsl(var(--primary))]/25">
              <MapPin class="h-6 w-6 text-[hsl(var(--primary-foreground))]" />
            </div>
            <div>
              <DialogTitle class="text-xl font-semibold text-[hsl(var(--card-foreground))] tracking-tight">
                添加 GPS 信息
              </DialogTitle>
              <DialogDescription class="text-[hsl(var(--muted-foreground))] text-sm mt-1">
                为选中的图像添加地理位置坐标
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>
      </div>

      <!-- 表单内容 - 增加 padding 和间距 -->
      <div class="px-8 pb-4 space-y-6">
        <!-- 纬度输入 -->
        <div class="space-y-3">
          <div class="flex items-center gap-2">
            <Compass class="h-4 w-4 text-[hsl(var(--primary))]" />
            <Label for="latitude" class="text-sm font-medium text-[hsl(var(--foreground))]">
              纬度 Latitude
            </Label>
          </div>
          <div class="relative">
            <Input
              id="latitude"
              v-model="latitude"
              type="text"
              placeholder="例如: 39.9042"
              :disabled="loading"
              class="h-12 px-4 bg-[hsl(var(--background))] border-[hsl(var(--input))] text-[hsl(var(--foreground))] placeholder:text-[hsl(var(--muted-foreground))] focus:border-[hsl(var(--ring))] focus:ring-[hsl(var(--ring))]/20 rounded-xl transition-all duration-200"
              :class="cn(
                latitude !== '' && !isValidLatitude && 'border-[hsl(var(--destructive))] focus:border-[hsl(var(--destructive))] focus:ring-[hsl(var(--destructive))]/20'
              )"
            />
            <div class="absolute right-4 top-1/2 -translate-y-1/2 text-sm text-[hsl(var(--muted-foreground))] font-medium">
              °N
            </div>
          </div>
          <p v-if="latitude !== '' && !isValidLatitude" class="text-sm text-[hsl(var(--destructive))] flex items-center gap-2">
            <span class="w-1.5 h-1.5 rounded-full bg-[hsl(var(--destructive))]" />
            纬度必须在 -90 到 90 之间
          </p>
        </div>

        <!-- 经度输入 -->
        <div class="space-y-3">
          <div class="flex items-center gap-2">
            <Navigation class="h-4 w-4 text-[hsl(var(--primary))]" />
            <Label for="longitude" class="text-sm font-medium text-[hsl(var(--foreground))]">
              经度 Longitude
            </Label>
          </div>
          <div class="relative">
            <Input
              id="longitude"
              v-model="longitude"
              type="text"
              placeholder="例如: 116.4074"
              :disabled="loading"
              class="h-12 px-4 bg-[hsl(var(--background))] border-[hsl(var(--input))] text-[hsl(var(--foreground))] placeholder:text-[hsl(var(--muted-foreground))] focus:border-[hsl(var(--ring))] focus:ring-[hsl(var(--ring))]/20 rounded-xl transition-all duration-200"
              :class="cn(
                longitude !== '' && !isValidLongitude && 'border-[hsl(var(--destructive))] focus:border-[hsl(var(--destructive))] focus:ring-[hsl(var(--destructive))]/20'
              )"
            />
            <div class="absolute right-4 top-1/2 -translate-y-1/2 text-sm text-[hsl(var(--muted-foreground))] font-medium">
              °E
            </div>
          </div>
          <p v-if="longitude !== '' && !isValidLongitude" class="text-sm text-[hsl(var(--destructive))] flex items-center gap-2">
            <span class="w-1.5 h-1.5 rounded-full bg-[hsl(var(--destructive))]" />
            经度必须在 -180 到 180 之间
          </p>
        </div>

        <!-- 提示信息卡片 -->
        <div class="relative overflow-hidden rounded-2xl bg-[hsl(var(--muted))]/50 border border-[hsl(var(--border))] p-5">
          <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-[hsl(var(--primary))]/60 via-[hsl(var(--accent))]/60 to-[hsl(var(--primary))]/60" />
          <div class="flex items-start gap-4">
            <div class="flex items-center justify-center w-10 h-10 rounded-xl bg-[hsl(var(--accent))]/20 shrink-0">
              <Globe class="h-5 w-5 text-[hsl(var(--primary))]" />
            </div>
            <div class="space-y-1.5 pt-0.5">
              <p class="text-sm font-medium text-[hsl(var(--foreground))]">
                查找坐标
              </p>
              <p class="text-sm text-[hsl(var(--muted-foreground))] leading-relaxed">
                访问
                <a
                  href="https://jingweidu.bmcx.com"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-[hsl(var(--primary))] hover:text-[hsl(var(--primary))]/80 transition-colors underline underline-offset-2 font-medium"
                >
                  经纬度查询网站
                </a>
                获取精确的地理位置坐标
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部按钮 - 增加 padding -->
      <DialogFooter class="px-8 pb-8 pt-4 gap-3">
        <Button
          variant="outline"
          :disabled="loading"
          @click="handleCancel"
          class="h-11 px-6 bg-transparent border-[hsl(var(--border))] text-[hsl(var(--foreground))] hover:bg-[hsl(var(--accent))] hover:text-[hsl(var(--accent-foreground))] transition-all duration-200 rounded-xl"
        >
          取消
        </Button>
        <Button
          :disabled="!canConfirm"
          @click="handleConfirm"
          class="h-11 px-6 bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))] hover:bg-[hsl(var(--primary))]/90 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 rounded-xl shadow-lg shadow-[hsl(var(--primary))]/20"
        >
          <Loader2 v-if="loading" class="mr-2 h-4 w-4 animate-spin" />
          <MapPin v-else class="mr-2 h-4 w-4" />
          <span>{{ loading ? '保存中...' : '确认添加' }}</span>
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
