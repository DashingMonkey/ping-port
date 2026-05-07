<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  cookies: Record<string, string>
}>()

const { t } = useI18n()
</script>

<template>
  <div class="text-xs">
    <div
      v-for="(value, key) in cookies"
      :key="key"
      class="flex py-1 border-b border-border-default last:border-b-0"
    >
      <span class="font-['JetBrains_Mono'] text-warning min-w-[120px] shrink-0 text-[11px]">{{ key }}</span>
      <span class="text-text-secondary break-all">{{ value }}</span>
    </div>
    <div v-if="Object.keys(cookies).length === 0" class="text-text-muted py-2 text-center">
      {{ t('response.noCookies') }}
    </div>
  </div>
</template>
