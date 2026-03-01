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
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2 text-destructive">
          <AlertTriangle class="h-5 w-5" />
          {{ title }}
        </DialogTitle>
        <DialogDescription>
          {{ message }}
        </DialogDescription>
      </DialogHeader>
      <DialogFooter class="gap-2 sm:gap-0">
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
