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
import { Button } from '../ui/button'
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
      class="sm:max-w-[440px] border overflow-hidden shadow-2xl p-0"
      :class="cn(
        'bg-[hsl(var(--card))] border-[hsl(var(--border))]',
        'dark:bg-[hsl(var(--card))] dark:border-[hsl(var(--border))]'
      )"
    >
      <!-- 顶部装饰区域 -->
      <div class="relative overflow-hidden">
        <!-- Light mode 渐变 -->
        <div class="absolute inset-0 bg-gradient-to-br from-[hsl(var(--destructive))]/10 via-transparent to-[hsl(var(--accent))]/10 dark:hidden" />
        <!-- Dark mode 渐变 -->
        <div class="absolute inset-0 bg-gradient-to-br from-[hsl(var(--destructive))]/20 via-transparent to-[hsl(var(--accent))]/15 hidden dark:block" />

        <!-- 装饰光晕 -->
        <div class="absolute -top-16 -right-16 w-32 h-32 bg-[hsl(var(--destructive))]/20 rounded-full blur-3xl dark:bg-[hsl(var(--destructive))]/15" />
        <div class="absolute -bottom-8 -left-8 w-24 h-24 bg-[hsl(var(--accent))]/15 rounded-full blur-2xl dark:bg-[hsl(var(--accent))]/10" />

        <DialogHeader class="relative z-10 px-8 pt-8 pb-6">
          <div class="flex items-center gap-4 mb-3">
            <div class="flex items-center justify-center w-12 h-12 rounded-2xl bg-[hsl(var(--destructive))] shadow-lg shadow-[hsl(var(--destructive))]/25">
              <AlertTriangle class="h-6 w-6 text-[hsl(var(--destructive-foreground))]" />
            </div>
            <div>
              <DialogTitle class="text-xl font-semibold text-[hsl(var(--card-foreground))] tracking-tight">
                {{ title }}
              </DialogTitle>
            </div>
          </div>
          <DialogDescription class="text-[hsl(var(--muted-foreground))] text-sm leading-relaxed">
            {{ message }}
          </DialogDescription>
        </DialogHeader>
      </div>

      <!-- 底部按钮 -->
      <DialogFooter class="px-8 pb-8 pt-2 gap-3">
        <Button
          variant="outline"
          @click="emit('cancel')"
          class="h-11 px-6 bg-transparent border-[hsl(var(--border))] text-[hsl(var(--foreground))] hover:bg-[hsl(var(--accent))] hover:text-[hsl(var(--accent-foreground))] transition-all duration-200 rounded-xl"
        >
          取消
        </Button>
        <Button
          v-for="opt in options"
          :key="opt.value"
          variant="destructive"
          @click="emit('confirm', opt.value)"
          class="h-11 px-6 bg-[hsl(var(--destructive))] text-[hsl(var(--destructive-foreground))] hover:bg-[hsl(var(--destructive))]/90 transition-all duration-200 rounded-xl shadow-lg shadow-[hsl(var(--destructive))]/20"
        >
          {{ opt.label }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
