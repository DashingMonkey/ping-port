<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import ResponseHeaders from './ResponseHeaders.vue'
import ResponseBody from './ResponseBody.vue'
import ResponseCookies from './ResponseCookies.vue'
import TestResults, { type TestResult } from './TestResults.vue'
import Console from './Console.vue'

const { t } = useI18n()

export interface HttpResponse {
  status: number
  status_text: string
  headers: Record<string, string>
  body: string
  time_ms: number
  size_bytes: number
}

const props = defineProps<{
  response: HttpResponse | null
  loading: boolean
  testResults?: TestResult[]
  consoleLogs?: string[]
  error?: string | null
}>()

type TabType = 'headers' | 'body' | 'cookies' | 'tests' | 'console'
const activeTab = ref<TabType>('body')

const cookies = computed<Record<string, string>>(() => {
  if (!props.response) return {}
  const setCookieHeader = props.response.headers['set-cookie'] || props.response.headers['Set-Cookie'] || ''
  if (!setCookieHeader) return {}

  const result: Record<string, string> = {}
  // Split on ", " followed by an attribute (expires, path, domain, etc.) not within the value
  const cookieParts = setCookieHeader.split(new RegExp(', (?=(?:expires|path|domain|secure|httponly|samesite))', 'i'))
  for (const part of cookieParts) {
    const [nameValue] = part.split(';')
    const eqIndex = nameValue.indexOf('=')
    if (eqIndex > 0) {
      const name = nameValue.substring(0, eqIndex).trim()
      const value = nameValue.substring(eqIndex + 1).trim()
      result[name] = value
    }
  }
  return result
})

function getStatusColor(status: number): string {
  if (status >= 200 && status < 300) return 'text-success'
  if (status >= 300 && status < 400) return 'text-warning'
  return 'text-error'
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Loading State -->
    <div v-if="loading" class="flex flex-col items-center justify-center h-full">
      <svg class="animate-spin h-8 w-8 text-accent mb-2" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
      <p class="text-xs text-text-muted">{{ t('response.sending') }}</p>
    </div>

    <!-- Response Content -->
    <template v-else-if="response">
      <!-- Tabs with status info on right -->
      <div class="flex items-center border-b border-border-default">
        <button
          @click="activeTab = 'body'"
          :class="[
            'px-3 py-1.5 text-xs font-medium transition-colors duration-150',
            activeTab === 'body'
              ? 'text-accent border-b-2 border-accent'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >
          {{ t('response.body') }}
        </button>
        <button
          @click="activeTab = 'cookies'"
          :class="[
            'px-3 py-1.5 text-xs font-medium transition-colors duration-150',
            activeTab === 'cookies'
              ? 'text-accent border-b-2 border-accent'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >
          {{ t('response.cookies') }}
        </button>
        <button
          @click="activeTab = 'headers'"
          :class="[
            'px-3 py-1.5 text-xs font-medium transition-colors duration-150',
            activeTab === 'headers'
              ? 'text-accent border-b-2 border-accent'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >
          {{ t('response.headers') }}
        </button>
        <button
          @click="activeTab = 'tests'"
          :class="[
            'px-3 py-1.5 text-xs font-medium transition-colors duration-150',
            activeTab === 'tests'
              ? 'text-accent border-b-2 border-accent'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >
          {{ t('response.testResults') }}
        </button>
        <button
          @click="activeTab = 'console'"
          :class="[
            'px-3 py-1.5 text-xs font-medium transition-colors duration-150',
            activeTab === 'console'
              ? 'text-accent border-b-2 border-accent'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >
          {{ t('response.console') }}
        </button>

        <!-- Status info on the right -->
        <div class="ml-auto flex items-center gap-3 pr-4">
          <span
            class="text-[10px] font-semibold"
            :class="getStatusColor(response.status)"
          >
            {{ response.status }} {{ response.status_text }}
          </span>
          <span class="text-[10px] text-text-muted">
            <span class="font-medium text-text-secondary">{{ response.time_ms }}ms</span>
          </span>
          <span class="text-[10px] text-text-muted">
            <span class="font-medium text-text-secondary">{{ formatBytes(response.size_bytes) }}</span>
          </span>
        </div>
      </div>

      <!-- Tab Content -->
      <div class="flex-1 min-h-0 flex flex-col overflow-hidden">
        <!-- Headers Tab -->
        <div v-show="activeTab === 'headers'" class="px-2 py-1 max-h-48 overflow-y-auto">
          <ResponseHeaders :headers="response.headers" />
        </div>

        <!-- Body Tab -->
        <div v-show="activeTab === 'body'" class="flex-1 p-2 min-h-0 overflow-auto">
          <ResponseBody :body="response.body" />
        </div>

        <!-- Cookies Tab -->
        <div v-show="activeTab === 'cookies'" class="px-2 py-1 max-h-48 overflow-y-auto">
          <ResponseCookies :cookies="cookies" />
        </div>

        <!-- Test Results Tab -->
        <div v-show="activeTab === 'tests'" class="flex-1 p-2 min-h-0 overflow-auto">
          <TestResults :results="testResults || []" />
        </div>

        <!-- Console Tab -->
        <div v-show="activeTab === 'console'" class="flex-1 min-h-0 overflow-hidden">
          <Console :logs="consoleLogs || []" />
        </div>
      </div>
    </template>

    <!-- Error State -->
    <div v-else-if="error" class="flex items-center justify-center h-full px-4">
      <div class="text-center max-w-lg">
        <div class="inline-flex items-center justify-center w-10 h-10 rounded-full bg-error/10 mb-2">
          <svg class="w-5 h-5 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        <p class="text-xs text-error font-medium mb-1">{{ t('response.requestFailed') }}</p>
        <p class="text-[10px] text-text-muted break-all">{{ error }}</p>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="flex items-center justify-center h-full">
      <p class="text-xs text-text-muted">{{ t('response.noResponse') }}</p>
    </div>
  </div>
</template>
