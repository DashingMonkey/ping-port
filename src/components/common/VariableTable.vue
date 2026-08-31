<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { toast } from '../../composables/useToast'

const { t } = useI18n()

const props = defineProps<{
  variables: Record<string, string>
  envId: string
}>()

const emit = defineEmits<{
  addVariable: [key: string, value: string]
  deleteVariable: [key: string]
  updateVariable: [oldKey: string, newKey: string, value: string]
}>()

interface FlatVariable {
  key: string
  value: string
}

const filteredVariables = computed<FlatVariable[]>(() => {
  return Object.entries(props.variables).map(([key, value]) => ({
    key,
    value,
  }))
})

// Persistent add-row state
const newKey = ref('')
const newValue = ref('')
const newKeyInputRef = ref<HTMLInputElement | null>(null)

// Handle key change - save on blur; reject empty and duplicate keys
function handleKeyChange(index: number, event: Event) {
  const input = event.target as HTMLInputElement
  const newKey = input.value.trim()
  const item = filteredVariables.value[index]
  if (!newKey) {
    input.value = item.key
    return
  }
  if (newKey !== item.key && props.variables[newKey] !== undefined) {
    toast.error(t('keyValue.duplicateKey'))
    input.value = item.key
    return
  }
  if (newKey !== item.key) {
    emit('updateVariable', item.key, newKey, item.value)
  }
}

// Handle value change - save on blur
function handleValueChange(index: number, newValue: string) {
  const item = filteredVariables.value[index]
  if (newValue !== item.value) {
    emit('updateVariable', item.key, item.key, newValue)
  }
}

// Save the persistent add-row (Enter)
function commitNew() {
  const key = newKey.value.trim()
  if (!key) {
    // Value without a key: keep everything, send focus back to the key input
    newKeyInputRef.value?.focus()
    return
  }
  if (props.variables[key] !== undefined) {
    toast.error(t('keyValue.duplicateKey'))
    newKeyInputRef.value?.focus()
    return
  }
  emit('addVariable', key, newValue.value)
  newKey.value = ''
  newValue.value = ''
  newKeyInputRef.value?.focus()
}

function clearNew() {
  newKey.value = ''
  newValue.value = ''
}
</script>

<template>
  <div>
    <!-- Header Row -->
    <div class="grid grid-cols-[1fr_1fr_36px] h-[26px] items-center border-b border-border-default sticky top-0 bg-surface-base z-10">
      <div class="px-2.5 text-[11px] text-text-muted">{{ t('keyValue.name') }}</div>
      <div class="px-2.5 text-[11px] text-text-muted">{{ t('keyValue.value') }}</div>
      <div></div>
    </div>

    <!-- Empty hint -->
    <div v-if="filteredVariables.length === 0" class="px-2.5 py-3 text-xs text-text-muted">
      {{ t('keyValue.noVariables') }}
    </div>

    <!-- Data Rows -->
    <div
      v-for="(item, index) in filteredVariables"
      :key="item.key"
      class="group grid grid-cols-[1fr_1fr_36px] h-7 items-center border-b border-border-default hover:bg-list-hover transition-colors"
    >
      <!-- Key column -->
      <div class="p-0.5">
        <input
          type="text"
          :value="item.key"
          @change="handleKeyChange(index, $event)"
          :placeholder="t('keyValue.name')"
          class="w-full h-[22px] px-1.5 text-[13px] bg-transparent border border-transparent rounded-[3px] outline-none text-text-primary placeholder-text-muted hover:border-border-default focus:bg-input-bg focus:border-border-focus transition-colors"
        />
      </div>

      <!-- Value column -->
      <div class="p-0.5">
        <input
          type="text"
          :value="item.value"
          @change="handleValueChange(index, ($event.target as HTMLInputElement).value)"
          :placeholder="t('keyValue.value')"
          class="w-full h-[22px] px-1.5 text-[12.5px] font-['JetBrains_Mono'] bg-transparent border border-transparent rounded-[3px] outline-none text-text-primary placeholder-text-muted hover:border-border-default focus:bg-input-bg focus:border-border-focus transition-colors"
        />
      </div>

      <!-- Delete column -->
      <div class="flex items-center justify-center">
        <button
          type="button"
          @click="emit('deleteVariable', item.key)"
          class="p-1 text-text-muted hover:text-error hover:bg-error/10 rounded opacity-0 group-hover:opacity-100 focus-visible:opacity-100 transition-opacity cursor-pointer"
          :title="t('keyValue.delete')"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Persistent add row -->
    <div class="grid grid-cols-[1fr_1fr_36px] h-7 items-center border-b border-border-default hover:bg-list-hover transition-colors">
      <div class="p-0.5">
        <input
          ref="newKeyInputRef"
          v-model="newKey"
          type="text"
          :placeholder="t('keyValue.addVariable')"
          class="w-full h-[22px] px-1.5 text-[13px] bg-transparent border border-transparent rounded-[3px] outline-none text-text-primary placeholder-text-muted hover:border-border-default focus:bg-input-bg focus:border-border-focus transition-colors"
          @keyup.enter="commitNew"
          @keyup.escape="clearNew"
        />
      </div>
      <div class="p-0.5">
        <input
          v-model="newValue"
          type="text"
          :placeholder="t('keyValue.value')"
          class="w-full h-[22px] px-1.5 text-[12.5px] font-['JetBrains_Mono'] bg-transparent border border-transparent rounded-[3px] outline-none text-text-primary placeholder-text-muted hover:border-border-default focus:bg-input-bg focus:border-border-focus transition-colors"
          @keyup.enter="commitNew"
          @keyup.escape="clearNew"
        />
      </div>
      <div></div>
    </div>
  </div>
</template>
