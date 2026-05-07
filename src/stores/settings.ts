import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import i18n from '../i18n'
import type { Language } from '../i18n'

export type Theme = 'light' | 'dark'
export type ActivityType = 'collections' | null

const SETTINGS_KEY = 'pingport-settings'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<Theme>('light')
  const sidebarCollapsed = ref(false)
  const activeActivity = ref<ActivityType>(null)
  const sidebarWidth = ref(256)
  const splitRatio = ref(55)
  const language = ref<Language>((i18n.global.locale as unknown as { value: string }).value.startsWith('zh') ? 'zh' : 'en')

  const loadSettings = () => {
    // Load all settings from localStorage (fast, synchronous)
    const stored = localStorage.getItem(SETTINGS_KEY)
    if (stored) {
      try {
        const parsed = JSON.parse(stored)
        if (parsed.theme) theme.value = parsed.theme
        if (typeof parsed.sidebarCollapsed === 'boolean') sidebarCollapsed.value = parsed.sidebarCollapsed
        if (parsed.activeActivity !== undefined) activeActivity.value = parsed.activeActivity
        if (parsed.sidebarWidth) sidebarWidth.value = parsed.sidebarWidth
        if (parsed.splitRatio) splitRatio.value = parsed.splitRatio
        if (parsed.language === 'en' || parsed.language === 'zh') {
          language.value = parsed.language
          ;(i18n.global.locale as unknown as { value: Language }).value = parsed.language
        }
      } catch {}
    }
    applyTheme()
  }

  // Save all settings to localStorage
  const saveSettings = () => {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify({
      theme: theme.value,
      sidebarCollapsed: sidebarCollapsed.value,
      activeActivity: activeActivity.value,
      sidebarWidth: sidebarWidth.value,
      splitRatio: splitRatio.value,
      language: language.value,
    }))
  }

  // Watch for changes and auto-save (debounced via immediate: false)
  watch([theme, sidebarCollapsed, activeActivity, sidebarWidth, splitRatio, language], () => {
    saveSettings()
  }, { flush: 'post' })

  const setLanguage = (lang: Language) => {
    language.value = lang
    ;(i18n.global.locale as unknown as { value: Language }).value = lang
  }

  const setTheme = (newTheme: Theme) => {
    theme.value = newTheme
    applyTheme()
  }

  const toggleTheme = () => {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
    applyTheme()
  }

  const setSidebarCollapsed = (collapsed: boolean) => {
    sidebarCollapsed.value = collapsed
  }

  const setActiveActivity = (activity: ActivityType) => {
    activeActivity.value = activity
  }

  const setSidebarWidth = (width: number) => {
    sidebarWidth.value = width
  }

  const setSplitRatio = (ratio: number) => {
    splitRatio.value = ratio
  }

  const applyTheme = () => {
    document.documentElement.classList.toggle('dark', theme.value === 'dark')
  }

  return {
    theme,
    sidebarCollapsed,
    activeActivity,
    sidebarWidth,
    splitRatio,
    language,
    loadSettings,
    setTheme,
    toggleTheme,
    setLanguage,
    setSidebarCollapsed,
    setActiveActivity,
    setSidebarWidth,
    setSplitRatio,
    applyTheme,
  }
})
