<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from '@headlessui/vue'
import { useWorkspaceStore } from '../../stores/workspace'
import { prompt } from '../../composables/usePrompt'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()

// Use name for selection to avoid object reference issues
const selectedWorkspaceName = computed(() => workspaceStore.currentWorkspace?.name)

async function handleSelect(name: string) {
  await workspaceStore.switchWorkspace(name)
  await workspaceStore.scanWorkspaces()
}

async function handleNewWorkspace() {
  const name = await prompt(t('workspace.enterName'), '', t('workspace.enterName'))
  if (name && name.trim()) {
    await invoke('create_workspace', { name: name.trim() })
    await workspaceStore.switchWorkspace(name.trim())
    await workspaceStore.scanWorkspaces()
  }
}

async function handleRename(e: Event, currentName: string) {
  e.stopPropagation()
  if (!currentName || currentName.startsWith('temp-')) return
  const newName = await prompt(t('workspace.renameTitle'), currentName, t('workspace.enterNewName'))
  if (newName && newName !== currentName) {
    await workspaceStore.renameWorkspace(currentName, newName)
    await workspaceStore.scanWorkspaces()
  }
}

const workspacesWithStatus = computed(() => {
  return workspaceStore.workspaces.map(w => ({
    ...w,
    isLocked: false,
  }))
})
</script>

<template>
  <Listbox
    :model-value="selectedWorkspaceName"
    @update:model-value="handleSelect"
  >
    <div class="relative">
      <!-- Toggle button -->
      <ListboxButton
        class="flex items-center gap-2 px-2 py-1 rounded-lg border
               bg-surface-base border-border-default
               hover:bg-surface-elevated transition-colors duration-150
               focus:outline-none min-w-[120px] cursor-pointer"
      >
        <!-- Workspace icon -->
        <svg class="w-4 h-4 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>

        <!-- Workspace label and name -->
        <span class="font-medium text-xs text-text-primary flex-1 text-left font-['IBM_Plex_Sans']">
          <span class="text-text-secondary">{{ t('workspace.label') }}</span>
          {{ workspaceStore.currentWorkspace?.name || t('workspace.select') }}
        </span>

        <!-- Dropdown icon -->
        <svg class="w-4 h-4 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </ListboxButton>

      <!-- Dropdown menu -->
      <ListboxOptions
        class="absolute z-50 mt-1 left-0 w-64 origin-top-left
               bg-surface-base border border-border-default rounded-lg shadow-xl
               max-h-96 overflow-auto"
      >
        <!-- Workspace list -->
        <div v-if="workspaceStore.workspaces.length > 0" class="p-2">
          <div class="text-[10px] font-semibold text-text-muted px-2 mb-1 uppercase tracking-wide">{{ t('workspace.switchWorkspace') }}</div>
          <ListboxOption
            v-for="ws in workspacesWithStatus"
            :key="ws.name"
            :value="ws.name"
            v-slot="{ active }"
          >
            <div
              :class="[
                'group flex items-center justify-between px-2 py-1.5 rounded-md transition-all duration-100 cursor-pointer',
                active ? 'bg-surface-elevated' : '',
                ws.name === selectedWorkspaceName ? 'bg-accent/10' : ''
              ]"
            >
              <div class="flex items-center gap-3 flex-1 min-w-0">
                <!-- Indicator -->
                <div
                  :class="[
                    'w-2 h-2 rounded-full flex-shrink-0',
                    ws.name === selectedWorkspaceName ? 'bg-accent' : 'bg-text-muted'
                  ]"
                />

                <!-- Name and status -->
                <div class="flex flex-col min-w-0">
                  <span :class="[
                    'font-medium text-xs truncate',
                    ws.name === selectedWorkspaceName ? 'text-accent' : 'text-text-primary'
                  ]">
                    {{ ws.name }}
                  </span>
                  <span v-if="ws.isTemporary" class="text-[10px] text-text-muted">{{ t('workspace.temporary') }}</span>
                </div>
              </div>

              <!-- Rename button (shown when selected) -->
              <button
                v-if="ws.name === selectedWorkspaceName && !ws.isTemporary"
                @click="(e) => handleRename(e, ws.name)"
                class="p-0.5 rounded text-text-muted hover:text-text-primary hover:bg-surface-base transition-colors opacity-0 group-hover:opacity-100"
                :title="t('workspace.rename')"
              >
                <svg class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                </svg>
              </button>
            </div>
          </ListboxOption>
        </div>

        <!-- Create new workspace -->
        <div class="border-t border-border-default p-2">
          <div class="flex items-center gap-2 px-2 py-1.5 text-xs text-accent hover:bg-surface-elevated rounded-md cursor-pointer transition-colors" @click="handleNewWorkspace">
            <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
            </svg>
            {{ t('workspace.new') }}
          </div>
        </div>
      </ListboxOptions>
    </div>
  </Listbox>
</template>

<style scoped>
/* Scrollbar styling for workspace list */
:deep(::-webkit-scrollbar) {
  width: 6px;
}

:deep(::-webkit-scrollbar-track) {
  background: transparent;
}

:deep(::-webkit-scrollbar-thumb) {
  background: var(--border-default);
  border-radius: 3px;
}

:deep(::-webkit-scrollbar-thumb:hover) {
  background: var(--text-muted);
}
</style>
