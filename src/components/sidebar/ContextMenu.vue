<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

export interface MenuItem {
  label: string
  action: () => void
  danger?: boolean
}

defineProps<{
  x: number
  y: number
  items: MenuItem[]
}>()

const emit = defineEmits<{
  close: []
}>()

const menuRef = ref<HTMLDivElement | null>(null)

const handleClickOutside = (event: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    emit('close')
  }
}

const handleItemClick = (item: MenuItem) => {
  item.action()
  emit('close')
}

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div
    ref="menuRef"
    class="fixed z-50 bg-surface-base rounded-lg shadow-xl border border-border-default py-1 min-w-[160px]"
    :style="{ left: `${x}px`, top: `${y}px` }"
  >
    <button
      v-for="(item, index) in items"
      :key="index"
      class="w-full px-4 py-1.5 text-left text-xs flex items-center gap-2 transition-colors duration-100 font-['IBM_Plex_Sans']"
      :class="[
        item.danger
          ? 'text-error hover:bg-error/10'
          : 'text-text-secondary hover:bg-surface-elevated hover:text-text-primary'
      ]"
      @click="handleItemClick(item)"
    >
      {{ item.label }}
    </button>
  </div>
</template>
