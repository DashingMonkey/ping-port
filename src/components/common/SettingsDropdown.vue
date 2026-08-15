<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Popover, PopoverButton, PopoverPanel } from '@headlessui/vue'
import { useI18n } from 'vue-i18n'
import { getVersion } from '@tauri-apps/api/app'
import { useSettingsStore } from '../../stores/settings'
import type { Language } from '../../i18n'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const appVersion = ref('')
onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = ''
  }
})

const languages: { value: Language; label: string }[] = [
  { value: 'en', label: 'English' },
  { value: 'zh', label: '中文' },
]

function setLanguage(lang: Language) {
  settingsStore.setLanguage(lang)
}
</script>

<template>
  <Popover class="relative" v-slot="{ open, close }">
    <PopoverButton
      class="flex items-center p-1.5 rounded hover:bg-surface-elevated text-text-secondary hover:text-text-primary transition-colors duration-150"
      :class="{ 'bg-surface-elevated text-text-primary': open }"
      :title="t('settings.title')"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </PopoverButton>

    <div v-if="open" class="fixed inset-0 z-40" @click="close()" />

    <PopoverPanel
      class="absolute z-50 right-0 mt-1 w-56 bg-surface-base border border-border-default rounded-lg shadow-xl p-3"
    >
      <div class="text-xs font-semibold text-text-primary mb-2">
        {{ t('settings.title') }}
      </div>
      <div class="flex items-center justify-between">
        <span class="text-[11px] text-text-secondary font-medium shrink-0">{{ t('settings.language') }}</span>
        <div class="flex items-center gap-0.5">
          <button
            v-for="lang in languages"
            :key="lang.value"
            @click="setLanguage(lang.value)"
            class="px-2 py-0.5 text-[11px] rounded transition-colors duration-150"
            :class="settingsStore.language === lang.value
              ? 'bg-accent/10 text-accent font-medium'
              : 'text-text-secondary hover:bg-surface-elevated hover:text-text-primary'"
          >
            {{ lang.label }}
          </button>
        </div>
      </div>

      <!-- Version -->
      <div v-if="appVersion" class="border-t border-border-default mt-2 pt-2 text-center">
        <span class="text-[10px] text-text-muted">v{{ appVersion }}</span>
      </div>
    </PopoverPanel>
  </Popover>
</template>
