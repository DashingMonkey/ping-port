import { h, render, shallowRef } from 'vue'
import Toast from '../components/common/Toast.vue'

export type ToastType = 'info' | 'success' | 'error' | 'warning'

export interface ToastOptions {
  message: string
  type?: ToastType
  duration?: number
}

interface ToastInstance {
  id: number
  container: HTMLElement
  timer: ReturnType<typeof setTimeout> | null
}

const toasts = shallowRef<ToastInstance[]>([])
let nextId = 0

function repositionToasts() {
  const els = document.querySelectorAll('.pp-toast')
  els.forEach((el, index) => {
    ;(el as HTMLElement).style.bottom = `${16 + index * 60}px`
  })
}

function showToast(options: ToastOptions): number {
  const id = nextId++
  const el = document.createElement('div')
  el.classList.add('pp-toast')
  el.style.position = 'fixed'
  el.style.right = '16px'
  el.style.zIndex = '9999'
  document.body.appendChild(el)

  let timer: ReturnType<typeof setTimeout> | null = null
  if (options.duration !== 0) {
    timer = setTimeout(() => {
      removeToast(id)
    }, options.duration || 3000)
  }

  const instance: ToastInstance = { id, container: el, timer }
  toasts.value = [...toasts.value, instance]

  const vnode = h(Toast, {
    message: options.message,
    type: options.type || 'info',
    duration: options.duration ?? 3000,
    onClose: () => removeToast(id),
  })
  render(vnode, el)
  repositionToasts()

  return id
}

function removeToast(id: number) {
  const instance = toasts.value.find(t => t.id === id)
  if (instance) {
    if (instance.timer) {
      clearTimeout(instance.timer)
    }
    render(null, instance.container)
    document.body.removeChild(instance.container)
    toasts.value = toasts.value.filter(t => t.id !== id)
    repositionToasts()
  }
}

// Convenience functions
function info(message: string, duration?: number) {
  return showToast({ message, type: 'info', duration })
}

function success(message: string, duration?: number) {
  return showToast({ message, type: 'success', duration })
}

function error(message: string, duration?: number) {
  return showToast({ message, type: 'error', duration })
}

function warning(message: string, duration?: number) {
  return showToast({ message, type: 'warning', duration })
}

// Singleton toast instance for simple usage
export const toast = {
  info,
  success,
  error,
  warning,
  showToast,
}

export function useToast() {
  return {
    showToast,
    info,
    success,
    error,
    warning,
    removeToast,
  }
}

export default toast
