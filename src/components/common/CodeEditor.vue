<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  modelValue: string
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const textarea = ref<HTMLTextAreaElement>()
const highlightDiv = ref<HTMLDivElement>()
const isDark = ref(false)

// Autocomplete state
const showSuggestions = ref(false)
const suggestions = ref<string[]>([])
const selectedIndex = ref(0)
const triggerPos = ref({ top: 0, left: 0 })
const triggerOffset = ref(0)
const suggestionType = ref<'variable' | 'pp'>('variable')

// Predefined variable list
const predefinedVars = [
  'timestamp',
  'uuid',
  'date',
  'time',
  'localeDate',
  'localeTime',
]

// pp method list
const ppMethods = [
  'variables',
  'request',
  'response',
  'test',
  'expect',
  'sleep',
]

// pp sub-methods
const ppSubMethods: Record<string, string[]> = {
  'variables': ['get', 'set', 'unset', 'replace'],
  'request': ['getUrl', 'setUrl', 'addQueryParam', 'removeQueryParam', 'hasQueryParam', 'getQueryParam'],
  'response': ['json', 'status', 'body', 'headers', 'time'],
}

// Detect theme
onMounted(() => {
  updateTheme()

  // Listen for storage events (cross-window theme switching)
  window.addEventListener('storage', (e) => {
    if (e.key === 'pingport-settings') {
      updateTheme()
    }
  })

  // Listen for DOM changes (same-window theme switching)
  const observer = new MutationObserver(() => {
    updateTheme()
  })
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] })

  document.addEventListener('click', handleClickOutside)

  onUnmounted(() => {
    observer.disconnect()
  })
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.code-editor')) {
    showSuggestions.value = false
  }
}

function updateTheme() {
  // Check document's dark class (main app switches theme via this)
  isDark.value = document.documentElement.classList.contains('dark')
}

const script = ref(props.modelValue)

// Calculate line count
const lineCount = computed(() => {
  const lines = script.value.split('\n').length
  return Math.max(lines, 1)
})

// Escape HTML
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

// Highlight function
function highlightCode(code: string): string {
  const escaped = escapeHtml(code)

  // Comments (match first)
  let result = escaped.replace(/(\/\/.*$)/gm, '<span class="comment">$1</span>')

  // Strings
  result = result.replace(/(&quot;[^&]*?&quot;|&#39;[^&]*?&#39;)/g, (match) => {
    return `<span class="string">${match}</span>`
  })

  // Numbers
  result = result.replace(/\b(\d+\.?\d*)\b/g, '<span class="number">$1</span>')

  // pp. API methods - match complete method names first (e.g., pp.variables.get, pp.request.addQueryParam)
  result = result.replace(/\b(pp\.(?:variables|request|response|test|expect|sleep)(?:\.(?:get|set|unset|replace|getUrl|setUrl|addQueryParam|removeQueryParam|hasQueryParam|getQueryParam|json|status|body|headers|time))?)\b/g, '<span class="pp-method">$1</span>')

  // Keywords
  result = result.replace(/\b(const|let|var|function|return|if|else|for|while|async|await|new|typeof|instanceof)\b/g, '<span class="keyword">$1</span>')

  // Variable references {{...}}
  result = result.replace(/\{\{([^}]*)\}\}/g, '<span class="variable">{{$1}}</span>')

  return result
}

// Highlight code
const highlightedCode = computed(() => highlightCode(script.value))

// Sync scroll
function syncScroll() {
  if (textarea.value && highlightDiv.value) {
    highlightDiv.value.scrollTop = textarea.value.scrollTop
    highlightDiv.value.scrollLeft = textarea.value.scrollLeft
  }
}

// Update on input
watch(script, (value) => {
  emit('update:modelValue', value)
})

watch(() => props.modelValue, (value) => {
  if (value !== script.value) {
    script.value = value
  }
})

// Detect {{ or . to trigger autocomplete
function handleInput() {
  const textareaEl = textarea.value
  if (!textareaEl) return

  const cursorPos = textareaEl.selectionStart
  const textBeforeCursor = script.value.substring(0, cursorPos)

  // Case 1: Detect {{
  const lastOpenBrace = textBeforeCursor.lastIndexOf('{{')
  if (lastOpenBrace !== -1) {
    const textBetween = textBeforeCursor.substring(lastOpenBrace)
    if (!textBetween.includes('}}')) {
      const query = textBeforeCursor.substring(lastOpenBrace + 2)

      const matched = predefinedVars.filter(v =>
        v.toLowerCase().startsWith(query.toLowerCase())
      )

      if (matched.length > 0) {
        const textAfterCursor = script.value.substring(cursorPos)
        const closingBraces = textAfterCursor.indexOf('}}')

        if (closingBraces === -1 || closingBraces > textAfterCursor.indexOf('{{', 2)) {
          suggestions.value = matched
          selectedIndex.value = 0
          showSuggestions.value = true
          triggerOffset.value = lastOpenBrace
          suggestionType.value = 'variable'
          updateSuggestionPosition(textareaEl, lastOpenBrace)
          return
        }
      }
    }
  }

  // Case 2: Detect pp. to trigger method completion
  // Find the last .
  const lastDot = textBeforeCursor.lastIndexOf('.')
  if (lastDot !== -1) {
    const textBeforeDot = textBeforeCursor.substring(0, lastDot)
    const afterDot = textBeforeCursor.substring(lastDot + 1)

    // Check if . is preceded by pp or a pp method name
    const charBeforeDot = textBeforeDot.slice(-2)

    // Case 2a: pp. triggers main method completion
    if (charBeforeDot.endsWith('pp') || textBeforeDot.endsWith('pp')) {
      // User typed after pp., filter method list
      const matched = ppMethods.filter(v =>
        v.toLowerCase().startsWith(afterDot.toLowerCase())
      )

      if (matched.length > 0 || afterDot === '') {
        suggestions.value = afterDot === '' ? ppMethods : matched
        selectedIndex.value = 0
        showSuggestions.value = true
        // triggerOffset points to position after .
        triggerOffset.value = lastDot + 1
        suggestionType.value = 'pp'
        updateSuggestionPosition(textareaEl, triggerOffset.value)
        return
      }
    }

    // Case 2b: pp.method. triggers sub-method completion
    // Check if beforeDot is a complete method name
    for (const method of ppMethods) {
      if (textBeforeDot.endsWith(method)) {
        // Preceded by complete method name, trigger sub-method completion
        const subMethods = ppSubMethods[method] || []
        if (subMethods.length > 0) {
          // Get user's sub-method prefix input
          const filtered = subMethods.filter(s =>
            afterDot === '' || s.toLowerCase().startsWith(afterDot.toLowerCase())
          )
          if (filtered.length > 0) {
            suggestions.value = filtered
            selectedIndex.value = 0
            showSuggestions.value = true
            // triggerOffset points to position after .
            triggerOffset.value = lastDot + 1
            suggestionType.value = 'pp'
            updateSuggestionPosition(textareaEl, triggerOffset.value)
            return
          }
        }
      }
    }
  }

  showSuggestions.value = false
}

function updateSuggestionPosition(textareaEl: HTMLTextAreaElement, triggerOffset: number) {
  const rect = textareaEl.getBoundingClientRect()

  // Get current line position
  const textBefore = script.value.substring(0, triggerOffset)
  const lines = textBefore.split('\n')
  const currentLine = lines.length
  const currentCol = lines[lines.length - 1].length

  // Calculate line height
  const lineHeight = 19.5

  // Calculate scroll offset
  const scrollTop = textareaEl.scrollTop
  const scrollLeft = textareaEl.scrollLeft

  // Estimate position
  triggerPos.value = {
    top: rect.top + (currentLine * lineHeight) - scrollTop + 20,
    left: rect.left + Math.min(currentCol * 7.8, rect.width - 150) - scrollLeft
  }
}

// Keyboard event handling
function handleKeydown(e: KeyboardEvent) {
  if (showSuggestions.value) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, suggestions.value.length - 1)
      return
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
      return
    }
    if (e.key === 'Enter' || e.key === 'Tab') {
      e.preventDefault()
      insertSuggestion(suggestions.value[selectedIndex.value])
      return
    }
    if (e.key === 'Escape') {
      showSuggestions.value = false
      return
    }
  }

  // Tab key inserts spaces
  if (e.key === 'Tab' && !showSuggestions.value) {
    e.preventDefault()
    const target = e.target as HTMLTextAreaElement
    const start = target.selectionStart
    const end = target.selectionEnd

    script.value = script.value.substring(0, start) + '  ' + script.value.substring(end)

    nextTick(() => {
      target.selectionStart = target.selectionEnd = start + 2
    })
  }

  // Ctrl+/ or Cmd+/ toggles comment on selected lines
  if ((e.ctrlKey || e.metaKey) && e.key === '/') {
    e.preventDefault()
    toggleComment()
  }
}

function toggleComment() {
  const target = textarea.value
  if (!target) return

  const text = script.value
  const start = target.selectionStart
  const end = target.selectionEnd

  // Find the start of the first line and end of the last line in the selection
  let lineStart = text.lastIndexOf('\n', start - 1) + 1
  let lineEnd = text.indexOf('\n', end - 1)
  if (lineEnd === -1) lineEnd = text.length

  // Include the trailing newline if selection ends exactly at a line start
  const selectedText = text.substring(lineStart, lineEnd)
  const lines = selectedText.split('\n')

  // Determine whether to comment or uncomment: comment if any line is not commented
  const allCommented = lines.every((line) => /^\s*\/\//.test(line))

  const newLines = lines.map((line) => {
    if (allCommented) {
      return line.replace(/^(\s*)\/\/ ?/, '$1')
    } else {
      return '// ' + line
    }
  })

  const newText = text.substring(0, lineStart) + newLines.join('\n') + text.substring(lineEnd)
  const lengthDiff = newText.length - text.length

  script.value = newText

  nextTick(() => {
    target.selectionStart = lineStart
    target.selectionEnd = lineEnd + lengthDiff
  })
}

function insertSuggestion(value: string) {
  const textareaEl = textarea.value
  if (!textareaEl) return

  const cursorPos = textareaEl.selectionStart
  const start = triggerOffset.value

  if (suggestionType.value === 'variable') {
    // Variable completion: replace {{ with {{value}}
    script.value =
      script.value.substring(0, start) +
      '{{' + value + '}}' +
      script.value.substring(cursorPos)

    showSuggestions.value = false

    nextTick(() => {
      const newPos = start + value.length + 4
      textareaEl.selectionStart = textareaEl.selectionEnd = newPos
      textareaEl.focus()
    })
  } else {
    // pp method completion: replace trigger point to current position with value
    // triggerOffset points to trigger point (after pp), cursorPos points to current cursor
    script.value =
      script.value.substring(0, start) +
      value +
      script.value.substring(cursorPos)

    showSuggestions.value = false

    nextTick(() => {
      // Place cursor after value
      textareaEl.selectionStart = textareaEl.selectionEnd = start + value.length
      textareaEl.focus()
    })
  }
}

function selectSuggestion(varName: string) {
  insertSuggestion(varName)
}
</script>

<template>
  <div class="code-editor" :class="{ dark: isDark }">
    <div class="editor-container">
      <!-- Line numbers -->
      <div class="line-numbers">
        <span v-for="i in lineCount" :key="i">{{ i }}</span>
      </div>

      <!-- Editor content -->
      <div class="editor-wrapper">
        <!-- Highlight layer -->
        <div
          ref="highlightDiv"
          class="highlight-layer"
          v-html="highlightedCode"
        ></div>

        <!-- Input layer -->
        <textarea
          ref="textarea"
          v-model="script"
          class="code-input"
          spellcheck="false"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          @scroll="syncScroll"
          @input="handleInput"
          @keydown="handleKeydown"
          :placeholder="placeholder"
        ></textarea>

        <!-- Autocomplete popup -->
        <div
          v-if="showSuggestions && suggestions.length > 0"
          class="suggestions-popup"
          :style="{
            top: triggerPos.top + 'px',
            left: triggerPos.left + 'px'
          }"
        >
          <div
            v-for="(suggestion, index) in suggestions"
            :key="suggestion"
            class="suggestion-item"
            :class="{ selected: index === selectedIndex }"
            @click="selectSuggestion(suggestion)"
            @mouseenter="selectedIndex = index"
          >
            {{ suggestion }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.code-editor {
  border: 1px solid #d1d5db;
  border-radius: 6px;
  background: #fff;
  overflow: hidden;
  font-family: 'JetBrains Mono', 'SF Mono', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 12px;
  line-height: 1.4;
  height: 120px;
  overflow-y: auto;
}

.code-editor.dark {
  border-color: #3c3c3c;
  background: #1e1e1e;
}

/* Container */
.editor-container {
  display: flex;
  min-height: 80px;
  position: relative;
}

/* Line numbers */
.line-numbers {
  width: 2.5rem;
  padding: 0.5rem 0.5rem;
  text-align: right;
  color: #6b7280;
  background: transparent;
  user-select: none;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.code-editor.dark .line-numbers {
  color: #6e7681;
  background: transparent;
}

/* Editor wrapper */
.editor-wrapper {
  flex: 1;
  position: relative;
  overflow: hidden;
}

/* Highlight layer */
.highlight-layer {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 0.5rem;
  pointer-events: none;
  white-space: pre-wrap;
  word-wrap: break-word;
  overflow: auto;
  color: #1a1a1a;
}

.code-editor.dark .highlight-layer {
  color: #d4d4d4;
}

/* Input layer */
.code-input {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 100%;
  padding: 0.5rem;
  border: none;
  outline: none;
  resize: none;
  background: transparent;
  color: transparent;
  caret-color: #1a1a1a;
  white-space: pre-wrap;
  word-wrap: break-word;
  font: inherit;
  z-index: 1;
}

.code-editor.dark .code-input {
  caret-color: #d4d4d4;
}

.code-input::placeholder {
  color: #9ca3af;
}

/* Fix ghosting when selecting commented text */
.code-input::selection {
  background-color: rgba(0, 0, 0, 0.3);
}

.code-editor.dark .code-input::selection {
  background-color: rgba(255, 255, 255, 0.2);
}

/* Autocomplete popup */
.suggestions-popup {
  position: fixed;
  background: #fff;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  max-height: 200px;
  overflow-y: auto;
  z-index: 1000;
  min-width: 120px;
}

.code-editor.dark .suggestions-popup {
  background: #2d2d2d;
  border-color: #3c3c3c;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.suggestion-item {
  padding: 4px 10px;
  cursor: pointer;
  font-size: 12px;
  color: #1a1a1a;
}

.code-editor.dark .suggestion-item {
  color: #d4d4d4;
}

.suggestion-item:hover,
.suggestion-item.selected {
  background: #0066cc;
  color: #fff;
}

.code-editor.dark .suggestion-item.selected {
  background: #4da6ff;
  color: #1e1e1e;
}

/* Syntax highlighting colors - VSCode Light+ */
.highlight-layer :deep(.keyword) {
  color: #0000ff;
}

.highlight-layer :deep(.pp-method) {
  color: #af00db;
}

.highlight-layer :deep(.string) {
  color: #a31515;
}

.highlight-layer :deep(.number) {
  color: #098658;
}

.highlight-layer :deep(.comment) {
  color: #008000;
}

.highlight-layer :deep(.variable) {
  color: #795e26;
  background: rgba(121, 94, 38, 0.1);
  padding: 0 2px;
  border-radius: 2px;
}

/* Syntax highlighting colors - VSCode Dark+ */
.code-editor.dark .highlight-layer :deep(.keyword) {
  color: #569cd6;
}

.code-editor.dark .highlight-layer :deep(.pp-method) {
  color: #dcdcaa;
}

.code-editor.dark .highlight-layer :deep(.string) {
  color: #ce9178;
}

.code-editor.dark .highlight-layer :deep(.number) {
  color: #b5cea8;
}

.code-editor.dark .highlight-layer :deep(.comment) {
  color: #6a9955;
}

.code-editor.dark .highlight-layer :deep(.variable) {
  color: #9cdcfe;
  background: rgba(156, 220, 254, 0.15);
}
</style>
