<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import JsonTreeView from './JsonTreeView.vue'

const { t } = useI18n()

const props = defineProps<{
  body: string
}>()

const viewMode = ref<'pretty' | 'raw' | 'preview'>('pretty')

function tryParseJson(str: string): object | null {
  try {
    const parsed = JSON.parse(str)
    if (typeof parsed === 'object' && parsed !== null) {
      return parsed
    }
    return null
  } catch {
    return null
  }
}

const parsedJson = computed(() => tryParseJson(props.body))
const isJson = computed(() => parsedJson.value !== null)

const isHtml = computed(() => {
  const trimmed = props.body.trim()
  return trimmed.startsWith('<!DOCTYPE') || trimmed.startsWith('<html') || trimmed.startsWith('<HTML')
})

const isXml = computed(() => {
  const trimmed = props.body.trim()
  return trimmed.startsWith('<?xml') || trimmed.startsWith('<')
})

const contentTypeKey = computed(() => {
  if (isJson.value) return 'json'
  if (isHtml.value) return 'html'
  if (isXml.value) return 'xml'
  return 'text'
})

const contentTypeBadgeClass = computed(() => {
  switch (contentTypeKey.value) {
    case 'json':
      return 'bg-success/15 text-success border-success/30'
    case 'html':
      return 'bg-warning/15 text-warning border-warning/30'
    case 'xml':
      return 'bg-info/15 text-info border-info/30'
    default:
      return 'bg-surface-elevated text-text-secondary border-border-default'
  }
})

const formattedBody = computed(() => {
  if (isJson.value) {
    return JSON.stringify(parsedJson.value, null, 2)
  }
  return props.body
})

const canPreview = computed(() => isHtml.value || isXml.value)

function setViewMode(mode: 'pretty' | 'raw' | 'preview') {
  viewMode.value = mode
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <span
          class="inline-flex items-center px-2 py-0.5 text-[11px] font-semibold rounded border"
          :class="contentTypeBadgeClass"
        >
          {{ t('response.' + contentTypeKey) }}
        </span>
      </div>
      <div class="flex items-center gap-1">
        <button
          type="button"
          @click="setViewMode('pretty')"
          class="inline-flex items-center px-2.5 py-1.5 text-[11px] font-medium rounded transition-colors duration-150"
          :class="viewMode === 'pretty'
            ? 'bg-accent/15 text-accent border border-accent/30'
            : 'bg-surface-elevated text-text-secondary hover:text-text-primary border border-border-default'"
        >
          <svg class="w-3.5 h-3.5 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16" />
          </svg>
          {{ t('response.pretty') }}
        </button>
        <button
          type="button"
          @click="setViewMode('raw')"
          class="inline-flex items-center px-2.5 py-1.5 text-[11px] font-medium rounded transition-colors duration-150"
          :class="viewMode === 'raw'
            ? 'bg-accent/15 text-accent border border-accent/30'
            : 'bg-surface-elevated text-text-secondary hover:text-text-primary border border-border-default'"
        >
          <svg class="w-3.5 h-3.5 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2 1 3 3 3h10c2 0 3-1 3-3V7c0-2-1-3-3-3H7c-2 0-3 1-3 3z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h8M8 8h8M8 16h5" />
          </svg>
          {{ t('response.raw') }}
        </button>
        <button
          v-if="canPreview"
          type="button"
          @click="setViewMode('preview')"
          class="inline-flex items-center px-2.5 py-1.5 text-[11px] font-medium rounded transition-colors duration-150"
          :class="viewMode === 'preview'
            ? 'bg-accent/15 text-accent border border-accent/30'
            : 'bg-surface-elevated text-text-secondary hover:text-text-primary border border-border-default'"
        >
          <svg class="w-3.5 h-3.5 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
          {{ t('response.preview') }}
        </button>
      </div>
    </div>

    <!-- Pretty Mode -->
    <div v-if="viewMode === 'pretty'" class="flex-1 overflow-auto">
      <JsonTreeView v-if="isJson" :data="parsedJson" />
      <pre v-else class="p-4 text-[13px] font-['JetBrains_Mono'] rounded border border-border-default bg-surface-deep whitespace-pre-wrap break-all text-text-secondary">{{ formattedBody }}</pre>
    </div>

    <!-- Raw Mode -->
    <pre
      v-if="viewMode === 'raw'"
      class="flex-1 p-4 text-[13px] font-['JetBrains_Mono'] rounded border border-border-default bg-surface-deep overflow-auto whitespace-pre-wrap break-all text-text-secondary"
    >{{ body }}</pre>

    <!-- Preview Mode (iframe for HTML/XML) -->
    <iframe
      v-if="viewMode === 'preview' && canPreview"
      :srcdoc="body"
      class="flex-1 w-full rounded border border-border-default bg-surface-deep"
      sandbox="allow-same-origin"
      :title="t('response.responsePreview')"
    />
  </div>
</template>
