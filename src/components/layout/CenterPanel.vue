<script setup lang="ts">
</script>

<template>
  <div class="center-panel">
    <slot />
  </div>
</template>

<style scoped>
.center-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-primary);
  overflow: hidden;
}
</style>
