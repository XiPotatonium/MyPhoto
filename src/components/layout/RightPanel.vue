<script setup lang="ts">
</script>

<template>
  <div class="right-panel">
    <slot />
  </div>
</template>

<style scoped>
.right-panel {
  width: 100%;
  height: 100%;
  background: var(--color-bg-secondary);
  overflow-y: auto;
  padding: var(--spacing-lg);
}
</style>
