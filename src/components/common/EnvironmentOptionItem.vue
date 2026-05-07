<script setup lang="ts">
import type { Environment } from '../../stores/environments'

defineProps<{
  env: Environment
  active: boolean
  selected: boolean
}>()
</script>

<template>
  <li
    :class="[
      'flex items-center justify-between px-2 py-1.5 cursor-pointer rounded transition-colors duration-100',
      active ? 'bg-surface-elevated text-text-primary' : 'text-text-secondary',
    ]"
  >
    <!-- Name -->
    <span class="font-medium truncate font-['IBM_Plex_Sans'] flex-1 text-xs">{{ env.name }}</span>

    <!-- Selected indicator -->
    <svg
      :class="['w-5 h-5 flex-shrink-0', selected ? 'text-accent' : 'text-transparent']"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
    </svg>
  </li>
</template>
