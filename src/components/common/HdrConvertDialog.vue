<script setup lang="ts">
import { computed } from 'vue'
import { AlertTriangle, Loader2, Sparkles } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog'
import { Button } from '../ui/button'
import type { HdrPlan } from '../../types/hdr'

const props = defineProps<{
  visible: boolean
  plan: HdrPlan | null
  loading: boolean
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const canConfirm = computed(() => !props.loading && (props.plan?.convertible ?? 0) > 0)

function previewNames(names: string[]): string {
  const preview = names.slice(0, 4).join('、')
  return names.length > 4 ? `${preview} 等 ${names.length} 个文件` : preview
}

function handleOpenChange(open: boolean) {
  // 转换确认后会立即关闭弹窗，这里只处理用户主动关闭（Esc / 点遮罩）
  if (!open && !props.loading) emit('cancel')
}
</script>

<template>
  <Dialog :open="visible" @update:open="handleOpenChange">
    <DialogContent
      :show-close-button="false"
      class="sm:max-w-[540px] p-0 overflow-hidden hdr-dialog"
    >
      <DialogHeader class="px-6 pt-5 pb-4">
        <div class="flex items-center gap-3">
          <div class="hdr-dialog-icon">
            <Sparkles class="h-5 w-5" />
          </div>
          <div class="flex flex-col gap-0.5">
            <DialogTitle>转换为 HDR</DialogTitle>
            <DialogDescription>
              把选中的 TIF / BMP 转换为同目录下的 HDR JPG（Ultra HDR），
              SDR 显示效果与原图保持一致
            </DialogDescription>
          </div>
        </div>
      </DialogHeader>

      <div class="hdr-dialog-body">
        <div v-if="loading" class="hdr-loading">
          <Loader2 class="h-5 w-5 animate-spin" />
          <span>正在检查选中的照片…</span>
        </div>

        <template v-else-if="plan">
          <div class="hdr-summary">
            <div class="hdr-summary-row">
              <span class="hdr-summary-label">选中照片</span>
              <span class="hdr-summary-value">{{ plan.total }} 张</span>
            </div>
            <div class="hdr-summary-row">
              <span class="hdr-summary-label">可以转换（TIF / BMP）</span>
              <span class="hdr-summary-value hdr-summary-value--ok">{{ plan.convertible }} 张</span>
            </div>
            <div v-if="plan.unsupported.length > 0" class="hdr-summary-row">
              <span class="hdr-summary-label">不支持转换</span>
              <span class="hdr-summary-value hdr-summary-value--muted">{{ plan.unsupported.length }} 张</span>
            </div>
          </div>

          <div v-if="plan.unsupported.length > 0" class="hdr-note">
            <p class="hdr-note-title">以下格式暂不支持转换：</p>
            <p class="hdr-note-list">{{ previewNames(plan.unsupported) }}</p>
          </div>

          <div v-if="plan.overwrite.length > 0" class="hdr-warning">
            <AlertTriangle class="hdr-warning-icon" />
            <div>
              <p class="hdr-warning-title">
                其中 {{ plan.overwrite.length }} 张图像已存在同名 JPG，转换后会覆盖原文件
              </p>
              <p class="hdr-warning-list">{{ previewNames(plan.overwrite) }}</p>
            </div>
          </div>

          <p class="hdr-tip">
            HDR JPG 将保存到原图同一目录，文件名后缀改为 .jpg。原 TIF / BMP 文件不会被修改。
          </p>
        </template>
      </div>

      <DialogFooter class="px-6 py-4 border-t border-border bg-muted/30">
        <Button variant="outline" :disabled="loading" @click="emit('cancel')">
          取消
        </Button>
        <Button :disabled="!canConfirm" @click="emit('confirm')">
          <Sparkles class="mr-2 h-4 w-4" />
          开始转换
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<style scoped>
.hdr-dialog-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: calc(var(--radius) - 2px);
  background: var(--primary);
  color: var(--primary-foreground);
}

.hdr-dialog-body {
  padding: 0 24px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.hdr-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 24px 0;
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
}

.hdr-summary {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: calc(var(--radius) - 2px);
  background: var(--secondary);
}

.hdr-summary-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  font-size: var(--font-size-sm);
}

.hdr-summary-label {
  color: var(--muted-foreground);
}

.hdr-summary-value {
  font-weight: var(--font-weight-semibold);
  font-variant-numeric: tabular-nums;
  color: var(--foreground);
}

.hdr-summary-value--ok {
  color: var(--primary);
}

.hdr-summary-value--muted {
  color: var(--muted-foreground);
}

.hdr-note {
  padding: 8px 12px;
  border-radius: calc(var(--radius) - 2px);
  background: var(--muted);
}

.hdr-note-title {
  font-size: var(--font-size-xs);
  color: var(--muted-foreground);
}

.hdr-note-list {
  margin-top: 2px;
  font-size: var(--font-size-xs);
  color: var(--foreground);
  word-break: break-all;
}

.hdr-warning {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--destructive);
  border-radius: calc(var(--radius) - 2px);
  background: oklch(from var(--destructive) l c h / 0.08);
}

.hdr-warning-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  margin-top: 1px;
  color: var(--destructive);
}

.hdr-warning-title {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  line-height: 1.4;
  color: var(--destructive);
}

.hdr-warning-list {
  margin-top: 3px;
  font-size: var(--font-size-xs);
  color: var(--destructive);
  word-break: break-all;
}

.hdr-tip {
  font-size: var(--font-size-xs);
  line-height: 1.5;
  color: var(--muted-foreground);
}
</style>
