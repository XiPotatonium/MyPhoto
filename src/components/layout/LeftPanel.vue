<script setup lang="ts">
import SplitPane from '../common/SplitPane.vue'
</script>

<template>
  <div class="left-panel">
    <SplitPane direction="vertical" :initial-ratio="40" :min-size="100">
      <template #first>
        <div class="panel-section">
          <slot name="tree" />
        </div>
      </template>
      <template #second>
        <div class="panel-section">
          <slot name="browser" />
        </div>
      </template>
    </SplitPane>
  </div>
</template>

<style scoped>
.left-panel {
  width: 100%;
  height: 100%;
  background: var(--secondary);
}

.panel-section {
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
