import { h, render, shallowRef } from 'vue'
import i18n from '../i18n'
import ConfirmDialog from '../components/common/ConfirmDialog.vue'

export interface ConfirmOptions {
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  danger?: boolean
}

const container = shallowRef<HTMLElement | null>(null)
let currentResolve: ((value: boolean) => void) | null = null

function showConfirm(options: ConfirmOptions): Promise<boolean> {
  return new Promise((resolve) => {
    // Resolve any pending promise first
    if (currentResolve) {
      currentResolve(false)
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

    const vnode = h(ConfirmDialog, {
      title: options.title ?? (options.danger ? i18n.global.t('common.confirmAction') : i18n.global.t('common.confirm')),
      confirmText: options.confirmText ?? i18n.global.t('common.confirm'),
      cancelText: options.cancelText ?? i18n.global.t('common.cancel'),
      message: options.message,
      danger: options.danger,
      onConfirm: () => {
        resolve(true)
        currentResolve = null
        cleanup()
      },
      onCancel: () => {
        resolve(false)
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

// Convenience functions
export function confirm(message: string, danger = false): Promise<boolean> {
  return showConfirm({ message, danger })
}

export function confirmDelete(message: string): Promise<boolean> {
  return showConfirm({
    message,
    confirmText: i18n.global.t('common.delete'),
    danger: true
  })
}

export default showConfirm
