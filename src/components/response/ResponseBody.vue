<script setup lang="ts">
import { ref, computed, watch } from 'vue'
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
  return trimmed.startsWith('<?xml') || (trimmed.startsWith('<') && !isHtml.value && trimmed.includes('</'))
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

// New response may not support the current mode (preview needs HTML/XML):
// reset to pretty so the body never renders as a blank area
watch(() => props.body, () => {
  viewMode.value = 'pretty'
})
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
      <div class="flex border border-border-default rounded-[3px] overflow-hidden">
        <button
          type="button"
          @click="setViewMode('pretty')"
          class="px-3 py-1 text-[11px] font-medium transition-colors duration-150 cursor-pointer"
          :class="viewMode === 'pretty'
            ? 'bg-list-active text-list-active-fg'
            : 'text-text-secondary hover:bg-list-hover hover:text-text-primary'"
        >
          {{ t('response.pretty') }}
        </button>
        <button
          type="button"
          @click="setViewMode('raw')"
          class="px-3 py-1 text-[11px] font-medium transition-colors duration-150 cursor-pointer border-l border-border-default"
          :class="viewMode === 'raw'
            ? 'bg-list-active text-list-active-fg'
            : 'text-text-secondary hover:bg-list-hover hover:text-text-primary'"
        >
          {{ t('response.raw') }}
        </button>
        <button
          v-if="canPreview"
          type="button"
          @click="setViewMode('preview')"
          class="px-3 py-1 text-[11px] font-medium transition-colors duration-150 cursor-pointer border-l border-border-default"
          :class="viewMode === 'preview'
            ? 'bg-list-active text-list-active-fg'
            : 'text-text-secondary hover:bg-list-hover hover:text-text-primary'"
        >
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
