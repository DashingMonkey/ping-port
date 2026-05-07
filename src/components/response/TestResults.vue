<script setup lang="ts">
import { useI18n } from 'vue-i18n'

export interface TestResult {
  name: string
  passed: boolean
  error?: string
  duration?: number
}

defineProps<{
  results: TestResult[]
}>()

const { t } = useI18n()

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}
</script>

<template>
  <div class="text-xs">
    <!-- Summary -->
    <div
      v-if="results.length > 0"
      class="px-3 py-2 border-b border-border-default bg-surface-elevated"
    >
      <span
        :class="[
          'inline-flex items-center px-2 py-0.5 text-[10px] font-semibold rounded border',
          results.every(r => r.passed)
            ? 'bg-success/15 text-success border-success/30'
            : 'bg-error/15 text-error border-error/30'
        ]"
      >
        {{ t('response.passed', { passed: results.filter(r => r.passed).length, total: results.length }) }}
      </span>
    </div>

    <!-- Results List -->
    <div class="divide-y divide-border-default">
      <div
        v-for="(result, index) in results"
        :key="index"
        class="px-3 py-2"
      >
        <div class="flex items-start gap-2">
          <!-- Pass/Fail Icon -->
          <div class="mt-0 shrink-0">
            <!-- Check icon for passed -->
            <svg
              v-if="result.passed"
              class="w-4 h-4 text-success"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 13l4 4L19 7"
              />
            </svg>
            <!-- X icon for failed -->
            <svg
              v-else
              class="w-4 h-4 text-error"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </div>

          <!-- Test Info -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between gap-2">
              <span class="font-medium text-text-primary truncate font-['IBM_Plex_Sans']">
                {{ result.name }}
              </span>
              <span
                v-if="result.duration !== undefined"
                class="text-[10px] text-text-muted shrink-0 font-['JetBrains_Mono']"
              >
                {{ formatDuration(result.duration) }}
              </span>
            </div>
            <p
              v-if="result.error"
              class="mt-1 text-[10px] text-error font-['JetBrains_Mono']"
            >
              {{ result.error }}
            </p>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div
        v-if="results.length === 0"
        class="px-3 py-4 text-center text-text-muted"
      >
        {{ t('response.noTestResults') }}
      </div>
    </div>
  </div>
</template>
