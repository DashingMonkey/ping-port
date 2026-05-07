<script setup lang="ts">
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSettingsStore } from '../../stores/settings'
import EnvironmentSwitcherDropdown from '../common/EnvironmentSwitcherDropdown.vue'
import EnvironmentManager from '../common/EnvironmentManager.vue'
import WorkspaceSelector from '../common/WorkspaceSelector.vue'
import SettingsDropdown from '../common/SettingsDropdown.vue'

const settingsStore = useSettingsStore()

const showEnvManager = ref(false)
const appWindow = getCurrentWindow()

const minimize = () => appWindow.minimize()
const toggleMaximize = () => appWindow.toggleMaximize()
const close = () => appWindow.close()
</script>

<template>
  <header class="h-10 flex items-center border-b border-border-default bg-surface-deep">
    <!-- Draggable Left Area (logo to right controls) -->
    <div class="flex items-center gap-3 px-4 flex-1" data-tauri-drag-region>
      <div class="flex items-center gap-2">
        <svg class="w-5 h-5 text-accent" viewBox="0 0 128 128" fill="currentColor">
          <rect x="8" y="8" width="112" height="112" rx="24" fill="currentColor" opacity="0.15"/>
          <circle cx="44" cy="48" r="12" fill="currentColor"/>
          <circle cx="84" cy="48" r="12" fill="currentColor"/>
          <rect x="32" y="76" width="64" height="24" rx="6" fill="currentColor"/>
        </svg>
        <span class="font-['IBM_Plex_Sans'] font-semibold tracking-tight text-text-primary">{{ $t('app.title') }}</span>
      </div>
      <WorkspaceSelector class="ml-4" style="-webkit-app-region: no-drag" />
    </div>

    <!-- Right Side Controls (non-draggable) -->
    <div class="flex items-center gap-2 pr-4">
      <EnvironmentSwitcherDropdown
        @open-manager="showEnvManager = true"
      />
      <SettingsDropdown />
      <button
        @click="settingsStore.toggleTheme()"
        class="relative flex items-center justify-between h-6 w-12 px-0.5 rounded-full bg-surface-elevated border border-border-default transition-all duration-200"
        title="Toggle theme"
      >
        <svg class="w-3.5 h-3.5 text-warning ml-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd"/>
        </svg>
        <svg class="w-3.5 h-3.5 text-text-muted mr-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"/>
        </svg>
        <span
          class="absolute top-1/2 -translate-y-1/2 w-5 h-5 rounded-full bg-surface-base border border-border-default shadow-sm transition-all duration-200"
          :class="settingsStore.theme === 'dark' ? 'left-[26px]' : 'left-[1px]'"
          style="transition-property: left, transform; will-change: left;"
        ></span>
      </button>

      <!-- Window Controls -->
      <div class="flex items-center ml-2 gap-1">
        <button
          @click="minimize"
          class="p-2 rounded hover:bg-surface-elevated text-text-secondary hover:text-text-primary transition-colors duration-150"
          :title="$t('window.minimize')"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path d="M3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z"/>
          </svg>
        </button>
        <button
          @click="toggleMaximize"
          class="p-2 rounded hover:bg-surface-elevated text-text-secondary hover:text-text-primary transition-colors duration-150"
          :title="$t('window.maximize')"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v12a1 1 0 01-1 1H4a1 1 0 01-1-1V4zm2 2v8h10V6H5z" clip-rule="evenodd"/>
          </svg>
        </button>
        <button
          @click="close"
          class="p-2 rounded hover:bg-error/20 text-text-secondary hover:text-error transition-colors duration-150"
          :title="$t('window.close')"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
          </svg>
        </button>
      </div>
    </div>
  </header>

  <!-- Environment Manager Modal -->
  <EnvironmentManager
    v-if="showEnvManager"
    @close="showEnvManager = false"
  />
</template>

<style scoped>
[data-tauri-drag-region] {
  -webkit-app-region: drag;
  cursor: grab;
}
</style>
