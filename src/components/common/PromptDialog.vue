<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  title: string
  message: string
  placeholder?: string
  defaultValue?: string
  confirmText: string
  cancelText: string
}>()

const emit = defineEmits<{
  confirm: [value: string]
  cancel: []
}>()

const isOpen = ref(true)
const inputValue = ref(props.defaultValue || '')

function handleConfirm() {
  if (inputValue.value.trim()) {
    isOpen.value = false
    emit('confirm', inputValue.value.trim())
  }
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

function handleKeydown(event: KeyboardEvent) {
  // Ignore keypresses while an IME (e.g. Chinese pinyin) is composing,
  // otherwise pressing Enter to confirm a candidate closes the dialog.
  if (event.isComposing) return
  if (event.key === 'Enter' && inputValue.value.trim()) {
    handleConfirm()
  } else if (event.key === 'Escape') {
    handleCancel()
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50"
      @click="handleOverlayClick"
    >
      <div class="bg-surface-base rounded-xl shadow-2xl w-full max-w-sm mx-4 overflow-hidden">
        <!-- Header -->
        <div class="px-4 pt-4 pb-1">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
            {{ title }}
          </h3>
        </div>

        <!-- Message -->
        <div class="px-4 py-3">
          <p class="text-xs text-gray-600 dark:text-gray-300 mb-2">{{ message }}</p>
          <input
            v-model="inputValue"
            type="text"
            :placeholder="placeholder"
            class="w-full px-3 py-1.5 text-xs border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:border-transparent"
            @keydown="handleKeydown"
            autofocus
          />
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
            :disabled="!inputValue.trim()"
            class="px-3 py-1.5 text-xs font-medium text-white rounded-lg transition-colors bg-blue-500 hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
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
