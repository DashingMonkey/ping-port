<script setup lang="ts">
defineProps<{
  title: string
  message: string
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center h-full p-6 text-center">
    <div class="text-accent/60 mb-4">
      <svg class="w-12 h-12 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
      </svg>
    </div>
    <h3 class="text-sm font-medium text-text-primary mb-1 font-['IBM_Plex_Sans']">{{ title }}</h3>
    <p class="text-xs text-text-muted font-['IBM_Plex_Sans']">{{ message }}</p>
  </div>
</template>
