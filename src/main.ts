import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import i18n from './i18n'
import './style.css'

const app = createApp(App)
app.use(i18n)
app.use(createPinia())

// Prevent default context menu; each component handles its own @contextmenu
document.addEventListener('contextmenu', (e) => {
  const target = e.target as HTMLElement
  // Only elements with @contextmenu bound are allowed to trigger
  if (!target.closest('[data-contextmenu]')) {
    e.preventDefault()
  }
})

app.mount('#app')
