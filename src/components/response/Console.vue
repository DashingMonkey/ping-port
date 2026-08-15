<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  logs: string[]
}>()

const { t } = useI18n()

const scrollRef = ref<HTMLElement | null>(null)

watch(() => props.logs, () => {
  nextTick(() => {
    if (scrollRef.value) {
      scrollRef.value.scrollTop = scrollRef.value.scrollHeight
    }
  })
}, { deep: true })
</script>

<template>
  <div class="h-full flex flex-col bg-white dark:bg-[#1e1e1e] text-gray-900 dark:text-gray-300 font-mono text-xs">
    <div ref="scrollRef" class="flex-1 overflow-auto p-2 space-y-0.5">
      <div v-if="logs.length === 0" class="text-gray-400 dark:text-gray-500 italic">
        {{ t('response.noConsole') }}
      </div>
      <div
        v-for="(log, index) in logs"
        :key="index"
        class="whitespace-pre-wrap break-all"
        :class="{
          'text-red-600 dark:text-red-400': log.startsWith('[ERROR]'),
          'text-yellow-600 dark:text-yellow-400': log.startsWith('[WARN]'),
          'text-green-600 dark:text-green-400': log.startsWith('[INFO]'),
          'text-blue-600 dark:text-blue-400': log.startsWith('[JS]'),
        }"
      >
        {{ log }}
      </div>
    </div>
  </div>
</template>
