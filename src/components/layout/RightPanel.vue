<script setup lang="ts">
</script>

<template>
  <div class="right-panel" @contextmenu.prevent>
    <slot />
  </div>
</template>

<style scoped>
.right-panel {
  width: 100%;
  height: 100%;
  background: var(--secondary);
  overflow-y: auto;
}
</style>
