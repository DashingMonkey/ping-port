<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

defineProps<{
  title: string
  message: string
  confirmText: string
  cancelText: string
  danger?: boolean
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const isOpen = ref(true)

function handleConfirm() {
  isOpen.value = false
  emit('confirm')
}

function handleCancel() {
  isOpen.value = false
  emit('cancel')
}

function handleOverlayClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    handleCancel()
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    handleCancel()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50"
        @click="handleOverlayClick"
      >
        <div class="bg-surface-base rounded-xl shadow-2xl w-full max-w-sm mx-4 overflow-hidden" role="dialog" aria-modal="true" aria-labelledby="pp-confirm-title">
          <!-- Header -->
          <div class="px-4 pt-4 pb-1 flex items-center gap-2">
            <div v-if="danger" class="w-8 h-8 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center flex-shrink-0">
              <svg class="w-4 h-4 text-red-600 dark:text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <h3 id="pp-confirm-title" class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {{ title }}
            </h3>
          </div>

          <!-- Message -->
          <div class="px-4 py-3">
            <p class="text-xs text-gray-600 dark:text-gray-300">{{ message }}</p>
          </div>

          <!-- Actions -->
          <div class="px-4 pb-4 flex gap-2 justify-end">
            <button
              @click="handleCancel"
              class="px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
            >
              {{ cancelText }}
            </button>
            <button
              @click="handleConfirm"
              class="px-3 py-1.5 text-xs font-medium text-white rounded-lg transition-colors"
              :class="danger
                ? 'bg-red-500 hover:bg-red-600 dark:bg-red-600 dark:hover:bg-red-700'
                : 'bg-blue-500 hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700'"
            >
              {{ confirmText }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.dialog-enter-active,
.dialog-leave-active {
  transition: opacity 0.2s ease;
}

.dialog-enter-from,
.dialog-leave-to {
  opacity: 0;
}

.dialog-enter-active > div,
.dialog-leave-active > div {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.dialog-enter-from > div,
.dialog-leave-to > div {
  transform: scale(0.95);
  opacity: 0;
}
</style>
