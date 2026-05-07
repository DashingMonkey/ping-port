import { createI18n } from 'vue-i18n'
import { en, zh } from './locales'

export type Language = 'en' | 'zh'

export function detectLanguage(): Language {
  try {
    const stored = localStorage.getItem('pingport-settings')
    if (stored) {
      const parsed = JSON.parse(stored)
      if (parsed.language === 'en' || parsed.language === 'zh') {
        return parsed.language
      }
    }
  } catch { /* corrupt localStorage, fall through */ }

  const raw = navigator.language
  if (raw.startsWith('zh')) return 'zh'
  return 'en'
}

const i18n = createI18n({
  legacy: false,
  locale: detectLanguage(),
  fallbackLocale: 'en',
  messages: { en, zh },
})

export default i18n
