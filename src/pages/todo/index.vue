<script setup lang="ts">
import TaskPanel from '@/components/task-panel/index.vue'
</script>

<template>
  <div class="h-screen w-screen bg-white dark:bg-#1a1a2e text-gray-800 dark:text-gray-200 overflow-hidden">
    <div class="flex flex-col h-full pt-2">
      <div class="flex-1 overflow-hidden">
        <TaskPanel />
      </div>
    </div>
  </div>
</template>