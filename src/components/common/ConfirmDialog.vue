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
      class="sm:max-w-[440px] p-0 overflow-hidden"
    >
      <!-- Header -->
      <DialogHeader class="px-6 pt-6 pb-4 flex flex-col gap-2">
        <div class="flex items-center gap-3">
          <div class="flex items-center justify-center w-10 h-10 rounded-md bg-destructive text-destructive-foreground shrink-0">
            <AlertTriangle class="h-5 w-5" />
          </div>
          <DialogTitle>{{ title }}</DialogTitle>
        </div>
        <DialogDescription class="ml-[52px] leading-relaxed">
          {{ message }}
        </DialogDescription>
      </DialogHeader>

      <!-- 底部按钮 -->
      <DialogFooter class="px-6 pb-6 pt-2">
        <Button variant="outline" @click="emit('cancel')">
          取消
        </Button>
        <Button
          v-for="opt in options"
          :key="opt.value"
          variant="destructive"
          @click="emit('confirm', opt.value)"
        >
          {{ opt.label }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
