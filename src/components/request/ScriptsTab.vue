<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import CodeEditor from '../common/CodeEditor.vue'

const { t } = useI18n()

const props = defineProps<{
  modelValue: {
    preRequestScript: string
    testScript: string
  }
}>()

const emit = defineEmits<{
  'update:modelValue': [value: { preRequestScript: string; testScript: string }]
}>()

type ScriptType = 'preRequest' | 'test'

const selectedScript = ref<ScriptType>('preRequest')

const scriptConfig = computed(() => ({
  preRequest: {
    label: t('scripts.preRequest'),
    helpText: t('scripts.preRequestHelp'),
    placeholder: '',
  },
  test: {
    label: t('scripts.tests'),
    helpText: t('scripts.testsHelp'),
    placeholder: '',
  },
}))

const currentScript = computed(() => scriptConfig.value[selectedScript.value])
const currentValue = computed(() =>
  selectedScript.value === 'preRequest'
    ? props.modelValue.preRequestScript
    : props.modelValue.testScript
)

function updateScript(value: string) {
  if (selectedScript.value === 'preRequest') {
    emit('update:modelValue', { ...props.modelValue, preRequestScript: value })
  } else {
    emit('update:modelValue', { ...props.modelValue, testScript: value })
  }
}

// Resize state (single editor now)
const editorRef = ref<InstanceType<typeof CodeEditor> | null>(null)
const isResizing = ref(false)
const startY = ref(0)
const startHeight = ref(120)

function startResize(event: MouseEvent) {
  isResizing.value = true
  startY.value = event.clientY
  const editorEl = editorRef.value?.$el as HTMLElement
  startHeight.value = editorEl?.clientHeight ?? 120
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
}

function onResize(event: MouseEvent) {
  if (!isResizing.value || !editorRef.value) return
  const delta = event.clientY - startY.value
  const newHeight = Math.max(60, startHeight.value + delta)
  const editorEl = editorRef.value?.$el as HTMLElement
  if (editorEl) {
    editorEl.style.height = `${newHeight}px`
  }
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
}

onUnmounted(() => {
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
})

async function openScriptsReferenceWindow() {
  const existing = await WebviewWindow.getByLabel('pp-scripts-reference')
  if (existing) {
    await existing.setFocus()
    return
  }

  const webview = new WebviewWindow('pp-scripts-reference', {
    url: '/pp-scripts-reference.html',
    title: t('scripts.scriptsReference'),
    width: 580,
    height: 600,
    resizable: true,
    decorations: true,
    alwaysOnTop: true,
  })

  webview.once('tauri://error', (e) => {
    console.error('WebviewWindow error:', e)
  })
}
</script>

<template>
  <div class="flex flex-row h-full">
    <!-- Left Menu -->
    <div class="w-36 border-r border-border-default flex flex-col pt-2 gap-1">
      <button
        v-for="(config, key) in scriptConfig"
        :key="key"
        class="group relative flex items-center gap-2 px-3 py-2 rounded mx-1 text-left transition-colors"
        :class="selectedScript === key ? 'bg-accent/10 text-accent' : 'hover:bg-bg-hover text-text-secondary hover:text-text-primary'"
        @click="selectedScript = key as ScriptType"
      >
        <!-- Radio indicator -->
        <span
          class="w-2 h-2 rounded-full border flex-shrink-0 transition-colors"
          :class="selectedScript === key ? 'border-accent bg-accent' : 'border-text-muted'"
        ></span>
        <span class="text-xs font-medium truncate">{{ config.label }}</span>

        <!-- Tooltip -->
        <div
          class="absolute left-full ml-2 top-1/2 -translate-y-1/2 px-2 py-1 bg-surface-elevated text-xs text-text-primary rounded shadow-lg whitespace-nowrap invisible group-hover:visible z-50 pointer-events-none border border-border"
        >
          {{ config.helpText }}
        </div>
      </button>

      <!-- Scripts Reference Button -->
      <button
        @click="openScriptsReferenceWindow"
        class="mx-2 mt-auto mb-2 text-[10px] text-accent hover:text-accent-hover transition-colors text-center px-3 py-1 whitespace-nowrap"
      >
        {{ t('scripts.showScriptsReference') }}
      </button>
    </div>

    <!-- Right Editor Panel -->
    <div class="flex-1 p-2 overflow-hidden flex flex-col">
      <div class="relative flex-1 flex flex-col overflow-y-auto min-h-0">
        <CodeEditor
          ref="editorRef"
          :model-value="currentValue"
          @update:model-value="updateScript"
          :placeholder="currentScript.placeholder"
          class="flex-1 min-h-0"
        />
        <!-- Resize handle -->
        <div
          class="absolute bottom-0 left-0 right-0 h-4 cursor-row-resize z-10"
          @mousedown.stop="startResize"
        ></div>
      </div>
    </div>
  </div>
</template>
