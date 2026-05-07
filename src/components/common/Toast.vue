<script setup lang="ts">
import { ref, onMounted } from 'vue'

const props = defineProps<{
  message: string
  type?: 'info' | 'success' | 'error' | 'warning'
  duration?: number
}>()

const emit = defineEmits<{
  close: []
}>()

const isVisible = ref(false)

onMounted(() => {
  // Trigger enter animation
  requestAnimationFrame(() => {
    isVisible.value = true
  })

  // Auto dismiss
  if (props.duration !== 0) {
    setTimeout(() => {
      handleClose()
    }, props.duration || 3000)
  }
})

function handleClose() {
  isVisible.value = false
  setTimeout(() => emit('close'), 200)
}

const icons = {
  info: `<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>`,
  success: `<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>`,
  error: `<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>`,
  warning: `<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg>`,
}

const colors = {
  info: 'bg-blue-50 dark:bg-blue-900 text-blue-800 dark:text-blue-200 border-blue-200 dark:border-blue-800',
  success: 'bg-green-50 dark:bg-green-900 text-green-800 dark:text-green-200 border-green-200 dark:border-green-800',
  error: 'bg-red-50 dark:bg-red-900 text-red-800 dark:text-red-200 border-red-200 dark:border-red-800',
  warning: 'bg-amber-50 dark:bg-amber-900 text-amber-800 dark:text-amber-200 border-amber-200 dark:border-amber-800',
}
</script>

<template>
  <div
    :class="[
      'flex items-center gap-2 px-3 py-2 rounded-lg border shadow-lg',
      'transition-all duration-200 ease-out',
      colors[type || 'info'],
      isVisible ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0'
    ]"
  >
    <span v-html="icons[type || 'info']" :class="colors[type || 'info']"></span>
    <span class="flex-1 text-xs font-medium">{{ message }}</span>
    <button
      @click="handleClose"
      class="p-0.5 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors"
    >
      <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
</template>
