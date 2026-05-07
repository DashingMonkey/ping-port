<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { RequestBody, KeyValuePair } from '../../stores/types'
import KeyValueTypeEditor from './KeyValueTypeEditor.vue'
import VariableTextarea from '../common/VariableTextarea.vue'

const { t } = useI18n()

const props = defineProps<{
  modelValue: RequestBody
  variables?: Record<string, string>
}>()

const emit = defineEmits<{
  'update:modelValue': [body: RequestBody]
}>()

const effectiveVars = computed(() => props.variables ?? {})

const bodyTypes = [
  { value: 'none', label: 'None' },
  { value: 'json', label: 'JSON' },
  { value: 'form-data', label: 'Form Data' },
  { value: 'x-www-form-urlencoded', label: 'x-www-form-urlencoded' }
] as const

const bodyLabelMap = computed(() => ({
  none: t('body.none'),
  json: t('body.json'),
  'form-data': t('body.formData'),
  'x-www-form-urlencoded': t('body.formUrlEncoded'),
}))

const prettifyError = ref<string | null>(null)

// Resize state
const textareaComponentRef = ref<InstanceType<typeof VariableTextarea> | null>(null)
const isResizing = ref(false)
const startY = ref(0)
const startHeight = ref(0)

function getTextareaEl(): HTMLDivElement | null | undefined {
  return textareaComponentRef.value?.containerRef
}

function startResize(event: MouseEvent) {
  isResizing.value = true
  startY.value = event.clientY
  startHeight.value = getTextareaEl()?.clientHeight ?? 192
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
}

function onResize(event: MouseEvent) {
  if (!isResizing.value) return
  const el = getTextareaEl()
  if (!el) return
  const delta = event.clientY - startY.value
  const newHeight = Math.max(80, startHeight.value + delta)
  el.style.height = `${newHeight}px`
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
}

function handleTypeChange(type: RequestBody['type']) {
  prettifyError.value = null
  // Clear body content when switching type
  if (type === 'none') {
    emit('update:modelValue', { type, content: '' })
  } else if (type === 'json') {
    emit('update:modelValue', { type, content: '' })
  } else if (type === 'form-data' || type === 'x-www-form-urlencoded') {
    emit('update:modelValue', { type, content: '[]' })
  }
}

function handleContentChange(content: string) {
  prettifyError.value = null
  emit('update:modelValue', { ...props.modelValue, content })
}

function prettify() {
  prettifyError.value = null
  if (props.modelValue.type !== 'json' || !props.modelValue.content.trim()) return

  try {
    const parsed = JSON.parse(props.modelValue.content)
    const formatted = JSON.stringify(parsed, null, 2)
    emit('update:modelValue', { ...props.modelValue, content: formatted })
  } catch (e) {
    prettifyError.value = t('body.invalidJson')
    setTimeout(() => {
      prettifyError.value = null
    }, 2000)
  }
}

// Parse form-data/urlencoded content to KeyValuePair[]
const formDataItems = computed<KeyValuePair[]>(() => {
  if (!props.modelValue.content.trim()) return []
  try {
    return JSON.parse(props.modelValue.content)
  } catch {
    return []
  }
})

function handleFormDataUpdate(items: KeyValuePair[]) {
  emit('update:modelValue', { ...props.modelValue, content: JSON.stringify(items) })
}
</script>

<template>
  <div class="p-2 space-y-2">
    <!-- Radio buttons row -->
    <div class="flex flex-wrap gap-x-4 gap-y-2 items-center">
      <label
        v-for="bt in bodyTypes"
        :key="bt.value"
        class="inline-flex items-center gap-2 cursor-pointer group"
      >
        <input
          type="radio"
          :value="bt.value"
          :checked="modelValue.type === bt.value"
          @change="handleTypeChange(bt.value)"
          class="w-4 h-4 border-border-default text-accent focus:ring-accent/50 focus:ring-offset-0 bg-surface-deep"
        />
        <span class="text-xs text-text-secondary group-hover:text-text-primary transition-colors duration-150">{{ bodyLabelMap[bt.value] }}</span>
      </label>
    </div>

    <!-- JSON body editor -->
    <div v-if="modelValue.type === 'json'" class="relative">
      <VariableTextarea
        ref="textareaComponentRef"
        :model-value="modelValue.content"
        :variables="effectiveVars"
        :placeholder="t('body.bodyPlaceholder')"
        class="w-full h-40"
        @update:model-value="handleContentChange"
      />
      <!-- Resize handle at bottom -->
      <div
        class="absolute bottom-0 left-0 right-0 h-4 cursor-row-resize"
        @mousedown.stop="startResize"
      ></div>
      <!-- Prettify button inside textarea -->
      <div v-if="modelValue.content.trim()" class="absolute bottom-2 right-2 flex items-center gap-2">
        <span v-if="prettifyError" class="text-[10px] text-error animate-pulse">{{ prettifyError }}</span>
        <button
          @click="prettify"
          class="inline-flex items-center gap-1 px-2 py-0.5 text-[10px] text-accent hover:text-cyan-300 bg-accent/10 hover:bg-accent/20 rounded transition-colors duration-150"
          :title="t('body.prettifyTooltip')"
        >
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16" />
          </svg>
          {{ t('body.prettify') }}
        </button>
      </div>
    </div>

    <!-- Form Data and x-www-form-urlencoded editor -->
    <div v-if="modelValue.type === 'form-data' || modelValue.type === 'x-www-form-urlencoded'">
      <KeyValueTypeEditor
        :model-value="formDataItems"
        :variables="effectiveVars"
        @update:model-value="handleFormDataUpdate"
      />
    </div>
  </div>
</template>
