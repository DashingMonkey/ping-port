<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const handleActivityClick = (activity: 'collections') => {
  if (settingsStore.activeActivity === activity) {
    // If already active, toggle off (collapse sidebar)
    settingsStore.setActiveActivity(null)
  } else {
    // Set as active and show sidebar
    settingsStore.setActiveActivity(activity)
  }
}
</script>

<template>
  <div class="w-12 h-full flex flex-col items-center py-2 gap-1 bg-surface-deep border-r border-border-default">
    <!-- Collections Button -->
    <button
      @click="handleActivityClick('collections')"
      class="w-10 h-10 flex items-center justify-center rounded-md transition-colors duration-150 group relative"
      :class="settingsStore.activeActivity === 'collections'
        ? 'bg-surface-active text-white'
        : 'text-text-secondary hover:bg-surface-elevated hover:text-text-primary'"
      :title="t(settingsStore.activeActivity === 'collections' ? 'sidebar.closeCollections' : 'sidebar.collections')"
    >
      <!-- Collections Icon -->
      <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>

      <!-- Tooltip on hover -->
      <div class="absolute left-full ml-2 px-2 py-1 bg-surface-elevated border border-border-default rounded text-xs text-text-primary whitespace-nowrap opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-150 z-50 pointer-events-none">
        {{ t(settingsStore.activeActivity === 'collections' ? 'sidebar.close' : 'sidebar.collections') }}
      </div>
    </button>

    <!-- Spacer -->
    <div class="flex-1"></div>
  </div>
</template>
