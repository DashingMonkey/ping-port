import { h, render, shallowRef } from 'vue'
import i18n from '../i18n'
import PromptDialog from '../components/common/PromptDialog.vue'

export interface PromptOptions {
  title?: string
  message: string
  placeholder?: string
  defaultValue?: string
  confirmText?: string
  cancelText?: string
}

const container = shallowRef<HTMLElement | null>(null)
let currentResolve: ((value: string | null) => void) | null = null

function showPrompt(options: PromptOptions): Promise<string | null> {
  return new Promise((resolve) => {
    // Resolve any pending promise first
    if (currentResolve) {
      currentResolve(null)
      currentResolve = null
    }

    // Remove existing dialog if any
    if (container.value) {
      render(null, container.value)
      document.body.removeChild(container.value)
    }

    // Create new container
    const el = document.createElement('div')
    document.body.appendChild(el)
    container.value = el

    currentResolve = resolve

    const vnode = h(PromptDialog, {
      title: options.title ?? i18n.global.t('common.enterValue'),
      confirmText: options.confirmText ?? i18n.global.t('common.confirm'),
      cancelText: options.cancelText ?? i18n.global.t('common.cancel'),
      message: options.message,
      placeholder: options.placeholder,
      defaultValue: options.defaultValue,
      onConfirm: (value: string) => {
        resolve(value)
        currentResolve = null
        cleanup()
      },
      onCancel: () => {
        resolve(null)
        currentResolve = null
        cleanup()
      }
    })
    render(vnode, el)
  })
}

function cleanup() {
  if (container.value) {
    render(null, container.value)
    document.body.removeChild(container.value)
    container.value = null
  }
}

// Convenience function
export function prompt(
  message: string,
  defaultValue?: string,
  placeholder?: string
): Promise<string | null> {
  return showPrompt({ message, defaultValue, placeholder })
}

export default showPrompt
