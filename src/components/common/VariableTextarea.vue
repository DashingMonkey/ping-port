<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const props = withDefaults(defineProps<{
  modelValue: string
  placeholder?: string
  variables: Record<string, string>
}>(), {
  placeholder: '',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const containerRef = ref<HTMLDivElement>()
const textareaRef = ref<HTMLTextAreaElement>()
const highlightRef = ref<HTMLDivElement>()
const isDark = ref(false)
const tooltipVisible = ref(false)
const tooltipText = ref('')
const tooltipStyle = ref({ top: '0px', left: '0px' })
const tooltipError = ref(false)
let activeVarEl: HTMLElement | null = null
let themeObserver: MutationObserver | null = null

defineExpose({ textareaRef, containerRef })

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

function highlightText(text: string): string {
  const escaped = escapeHtml(text)
  return escaped.replace(
    /\{\{([^}]+)\}\}/g,
    (_match, varName: string) => {
      const trimmed = varName.trim()
      const found = trimmed.length > 0 && props.variables[trimmed] !== undefined
      const cls = found ? 'var-found' : 'var-not-found'
      return `{{<span class="${cls}" data-varname="${escapeHtml(trimmed)}">${escapeHtml(trimmed)}</span>}}`
    },
  )
}

const highlightedHtml = computed(() => {
  if (!props.modelValue) return ''
  return highlightText(props.modelValue)
})

function handleInput(e: Event) {
  const target = e.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}

function syncScroll() {
  if (textareaRef.value && highlightRef.value) {
    highlightRef.value.scrollTop = textareaRef.value.scrollTop
    highlightRef.value.scrollLeft = textareaRef.value.scrollLeft
  }
}

function getVarSpan(el: HTMLElement): HTMLElement | null {
  if (el.classList.contains('var-found') || el.classList.contains('var-not-found')) return el
  const parent = el.parentElement
  if (parent && (parent.classList.contains('var-found') || parent.classList.contains('var-not-found'))) {
    return parent
  }
  return null
}

function showTooltip(el: HTMLElement) {
  const varName = el.dataset.varname || ''
  const value = props.variables[varName]
  if (value !== undefined) {
    tooltipText.value = t('variables.found', { name: varName, value })
    tooltipError.value = false
  } else {
    tooltipText.value = t('variables.notFound', { name: varName })
    tooltipError.value = true
  }
  const rect = el.getBoundingClientRect()
  tooltipStyle.value = {
    top: `${rect.bottom + 4}px`,
    left: `${Math.max(0, rect.left)}px`,
  }
  tooltipVisible.value = true
}

function hideTooltip() {
  tooltipVisible.value = false
  activeVarEl = null
}

// Highlight layer is on TOP of the textarea; var spans have pointer-events: auto.
function onHighlightMouseOver(e: MouseEvent) {
  const target = e.target as HTMLElement
  const span = getVarSpan(target)
  if (!span) {
    hideTooltip()
    return
  }
  if (span === activeVarEl) return
  activeVarEl = span
  showTooltip(span)
}

function onHighlightMouseOut(e: MouseEvent) {
  const target = e.target as HTMLElement
  const span = getVarSpan(target)
  if (!span) return
  const related = e.relatedTarget as HTMLElement | null
  if (related && (getVarSpan(related) === span || span.contains(related))) return
  hideTooltip()
}

function onHighlightClick(e: MouseEvent) {
  const span = getVarSpan(e.target as HTMLElement)
  if (span) {
    textareaRef.value?.focus()
  }
}

onMounted(() => {
  const html = document.documentElement
  isDark.value = html.classList.contains('dark')

  themeObserver = new MutationObserver(() => {
    isDark.value = html.classList.contains('dark')
  })
  themeObserver.observe(html, { attributes: true, attributeFilter: ['class'] })
})

onUnmounted(() => {
  themeObserver?.disconnect()
  themeObserver = null
})
</script>

<template>
  <div ref="containerRef" class="var-textarea-container" :class="{ dark: isDark }">
    <!-- Textarea behind (z-index: 0), highlight on top (z-index: 1) -->
    <textarea
      ref="textareaRef"
      :value="modelValue"
      :placeholder="placeholder"
      @input="handleInput"
      @scroll="syncScroll"
      class="var-textarea-field"
      spellcheck="false"
      autocomplete="off"
    ></textarea>
    <div
      ref="highlightRef"
      class="var-textarea-highlight"
      aria-hidden="true"
      @mouseover="onHighlightMouseOver"
      @mouseout="onHighlightMouseOut"
      @click="onHighlightClick"
    >
      <!-- SAFETY: highlightedHtml is built by highlightText() which calls escapeHtml() on user text before injecting any HTML -->
      <span v-if="highlightedHtml" v-html="highlightedHtml"></span>
    </div>
    <Teleport to="body">
      <div
        v-if="tooltipVisible"
        class="var-tooltip"
        :class="{ 'var-tooltip-error': tooltipError, dark: isDark }"
        :style="tooltipStyle"
      >
        {{ tooltipText }}
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.var-textarea-container {
  position: relative;
  width: 100%;
  /* height is set by the parent via Tailwind class (e.g. h-40) or inline style (resize handle) */
}

.var-textarea-highlight {
  position: absolute;
  inset: 0;
  z-index: 1;
  padding: 0.5rem 0.75rem;
  font-family: 'JetBrains Mono', 'SF Mono', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 11px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
  overflow: auto;
  pointer-events: none;
  user-select: none;
  -webkit-user-select: none;
  color: var(--text-primary);
}

.var-textarea-field {
  position: absolute;
  inset: 0;
  z-index: 0;
  width: 100%;
  height: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border-default);
  border-radius: 0.25rem;
  outline: none;
  resize: none;
  background: var(--surface-deep);
  color: transparent;
  caret-color: var(--text-primary);
  font-family: 'JetBrains Mono', 'SF Mono', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 11px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.var-textarea-container.dark .var-textarea-field {
  caret-color: #d4d4d4;
}

.var-textarea-field:focus {
  border-color: var(--accent);
}

.var-textarea-field::placeholder {
  color: var(--text-muted);
}

.var-textarea-field::selection {
  background-color: rgba(0, 0, 0, 0.25);
  color: transparent;
  -webkit-text-fill-color: transparent;
}

.var-textarea-container.dark .var-textarea-field::selection {
  background-color: rgba(255, 255, 255, 0.15);
  color: transparent;
  -webkit-text-fill-color: transparent;
}

/* Variable name — found. Blue in both themes, no background. pointer-events: auto to detect hover. */
.var-textarea-highlight :deep(.var-found) {
  color: #0066cc;
  pointer-events: auto;
  cursor: default;
}

/* Variable name — not found. Red in both themes, no background. */
.var-textarea-highlight :deep(.var-not-found) {
  color: #e51400;
  pointer-events: auto;
  cursor: default;
}

.var-textarea-container.dark .var-textarea-highlight :deep(.var-not-found) {
  color: #f14c4c;
}

/* Tooltip (teleported to body) — light theme default */
.var-tooltip {
  position: fixed;
  background: #e8e8e8;
  color: #1a1a1a;
  padding: 3px 8px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', 'SF Mono', Consolas, monospace;
  font-size: 10px;
  line-height: 1.4;
  pointer-events: none;
  z-index: 9999;
  white-space: nowrap;
  border: 1px solid #d1d1d1;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

/* Dark theme overrides */
.var-tooltip.dark {
  background: #1e1e1e;
  color: #cccccc;
  border-color: #3c3c3c;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.var-tooltip-error {
  color: #d41400;
}

.var-tooltip.dark.var-tooltip-error {
  color: #f14c4c;
  border-color: #5a1a1a;
}
</style>
