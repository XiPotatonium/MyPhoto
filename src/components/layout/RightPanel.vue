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
  background: hsl(var(--secondary));
  overflow-y: auto;
}
</style>
