<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { cn } from "@/lib/utils"

interface Props {
  class?: HTMLAttributes["class"]
}

const props = defineProps<Props>()
</script>

<template>
  <div
    data-slot="button-group"
    :class="cn(
      'inline-flex items-center justify-center rounded-md bg-muted p-1',
      props.class
    )"
  >
    <slot />
  </div>
</template>
