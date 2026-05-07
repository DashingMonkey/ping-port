<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from '@headlessui/vue'
import { useEnvironmentsStore, type Environment } from '../../stores/environments'
import { useTabsStore } from '../../stores/tabs'
import { useRequestsStore } from '../../stores/requests'
import EnvironmentOptionItem from './EnvironmentOptionItem.vue'

const { t } = useI18n()

const emit = defineEmits<{
  openManager: []
}>()

const environmentsStore = useEnvironmentsStore()
const tabsStore = useTabsStore()
const requestsStore = useRequestsStore()

const selectedEnv = computed({
  get: () => {
    const activeTab = tabsStore.activeTab
    const tabId = activeTab?.id ?? null
    let collectionId: string | null = null
    if (activeTab?.requestId) {
      const request = requestsStore.requests.find(r => r.id === activeTab.requestId)
      collectionId = request?.collectionId ?? null
    }
    const envId = environmentsStore.resolveActiveEnvId(tabId, collectionId)
    return environmentsStore.getEnvironmentById(envId) ?? null
  },
  set: (env: Environment | null) => {
    if (env) {
      environmentsStore.setActiveEnvironment(env.id)
    }
  }
})

const environments = computed(() => environmentsStore.environments.filter(e => e.id !== '__global__'))
</script>

<template>
  <Listbox
    :model-value="selectedEnv"
    @update:model-value="(env) => env && environmentsStore.setActiveEnvironment(env.id)"
  >
    <div class="relative">
      <!-- Toggle button -->
      <ListboxButton
        class="group flex items-center gap-2 px-2 py-1 rounded-lg border
               bg-surface-base border-border-default
               hover:bg-surface-elevated transition-colors duration-150
               focus:outline-none min-w-[120px]"
      >
        <!-- Environment name -->
        <span class="font-medium text-xs text-text-primary flex-1 text-left font-['IBM_Plex_Sans']">
          {{ selectedEnv?.name || t('environment.noEnvironment') }}
        </span>

        <!-- Clear button -->
        <button
          v-if="selectedEnv"
          @click.stop="environmentsStore.setActiveEnvironment(null)"
          class="p-0.5 rounded hover:bg-surface-elevated text-text-muted hover:text-text-primary transition-colors duration-150 opacity-0 group-hover:opacity-100"
          :title="t('environment.clearSelection')"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </ListboxButton>

      <!-- Dropdown menu -->
      <ListboxOptions
        class="absolute z-50 mt-1 right-0 w-56 bg-surface-base
               border border-border-default rounded-lg shadow-xl
               max-h-96 overflow-auto"
      >
        <!-- All environments -->
        <div v-if="environments.length > 0" class="p-2">
          <div class="text-[10px] font-semibold text-text-muted px-2 mb-1 uppercase tracking-wide">{{ t('environment.allEnvironments') }}</div>
          <ListboxOption
            v-for="env in environments"
            :key="env.id"
            :value="env"
            v-slot="{ active, selected }"
          >
            <EnvironmentOptionItem
              :env="env"
              :active="active"
              :selected="selected"
            />
          </ListboxOption>
        </div>

        <!-- Bottom actions -->
        <div class="border-t border-border-default p-2">
          <button
            @click="emit('openManager')"
            class="flex items-center gap-2 w-full px-2 py-1.5 text-xs
                   text-text-secondary hover:text-text-primary hover:bg-surface-elevated
                   rounded transition-colors duration-150 font-['IBM_Plex_Sans']"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            {{ t('environment.manageEnvironments') }}
          </button>
        </div>
      </ListboxOptions>
    </div>
  </Listbox>
</template>
