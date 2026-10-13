<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  headers: Record<string, string[]>
}>()

const { t } = useI18n()
</script>

<template>
  <div class="text-xs">
    <table class="w-full">
      <tbody>
        <template v-for="(values, key) in headers" :key="key">
          <tr v-for="(value, index) in values" :key="index">
            <td class="font-['JetBrains_Mono'] text-accent text-[11px] pr-4 py-1 align-top whitespace-nowrap">{{ key }}</td>
            <td class="text-text-secondary py-1 align-top break-all">{{ value }}</td>
          </tr>
        </template>
      </tbody>
    </table>
    <div v-if="Object.keys(headers).length === 0" class="text-text-muted py-2 text-center">
      {{ t('response.noHeaders') }}
    </div>
  </div>
</template>
