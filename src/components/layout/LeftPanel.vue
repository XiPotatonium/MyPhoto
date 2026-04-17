<script setup lang="ts">
</script>

<template>
  <div class="left-panel">
    <slot />
  </div>
</template>

<style scoped>
.left-panel {
  width: 100%;
  height: 100%;
  background: var(--secondary);
  overflow: hidden;
}
</style>
