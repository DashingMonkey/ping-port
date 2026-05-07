<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import VariableInput from '../common/VariableInput.vue'

const { t } = useI18n()

interface KeyValuePair {
  key: string
  value: string
  enabled: boolean
  type?: 'text' | 'file'
}

const props = defineProps<{
  modelValue: KeyValuePair[]
  variables?: Record<string, string>
}>()

const emit = defineEmits<{
  'update:modelValue': [items: KeyValuePair[]]
}>()

const effectiveVars = computed(() => props.variables ?? {})

let nextId = 0

function rowKey(item: KeyValuePair): number {
  const ext = item as KeyValuePair & { _uid?: number }
  if (ext._uid === undefined) {
    ext._uid = ++nextId
  }
  return ext._uid
}

function handleEnabledChange(index: number, enabled: boolean) {
  const newItems = [...props.modelValue]
  newItems[index] = { ...newItems[index], enabled }
  emit('update:modelValue', newItems)
}

function handleKeyChange(index: number, key: string) {
  const newItems = [...props.modelValue]
  newItems[index] = { ...newItems[index], key }
  emit('update:modelValue', newItems)
}

function handleValueChange(index: number, value: string) {
  const newItems = [...props.modelValue]
  newItems[index] = { ...newItems[index], value }
  emit('update:modelValue', newItems)
}

function handleTypeChange(index: number, type: 'text' | 'file') {
  const newItems = [...props.modelValue]
  newItems[index] = { ...newItems[index], type }
  emit('update:modelValue', newItems)
}

function handleDelete(index: number) {
  const newItems = props.modelValue.filter((_: KeyValuePair, i: number) => i !== index)
  emit('update:modelValue', newItems)
}

function handleAdd() {
  const newItems = [...props.modelValue, { key: '', value: '', enabled: true, type: 'text' as const }]
  emit('update:modelValue', newItems)
}
</script>

<template>
  <div class="space-y-0">
    <!-- Table wrapper with full border -->
    <div class="border border-border-default rounded">
      <!-- Header Row -->
      <div class="grid grid-cols-[28px_1fr_1fr_80px_40px] divide-x divide-border-default bg-surface-base rounded-t">
        <div class="flex items-center justify-center h-6">
          <span class="w-4 h-4 flex-shrink-0"></span>
        </div>
        <div class="px-3 flex items-center h-6">
          <span class="text-[11px] font-semibold tracking-wide text-text-muted uppercase">{{ t('keyValue.key') }}</span>
        </div>
        <div class="px-3 flex items-center h-6">
          <span class="text-[11px] font-semibold tracking-wide text-text-muted uppercase">{{ t('keyValue.value') }}</span>
        </div>
        <div class="px-3 flex items-center h-6">
          <span class="text-[11px] font-semibold tracking-wide text-text-muted uppercase">{{ t('keyValue.type') }}</span>
        </div>
        <div class="flex items-center justify-center h-6">
          <span class="w-4"></span>
        </div>
      </div>

      <!-- Data Rows -->
      <div
        v-for="(item, index) in modelValue"
        :key="rowKey(item)"
        class="grid grid-cols-[28px_1fr_1fr_80px_40px] divide-x divide-border-default border-t border-border-default group"
      >
        <div class="flex items-center justify-center h-6">
          <input
            type="checkbox"
            :checked="item.enabled"
            @change="handleEnabledChange(index, ($event.target as HTMLInputElement).checked)"
            class="w-4 h-4 rounded border-border-default text-accent focus:border-accent focus:ring-offset-0 bg-surface-deep"
          />
        </div>

        <VariableInput
          :model-value="item.key"
          :variables="effectiveVars"
          :disabled="!item.enabled"
          :placeholder="t('keyValue.key')"
          @update:model-value="handleKeyChange(index, $event)"
        />

        <VariableInput
          :model-value="item.value"
          :variables="effectiveVars"
          :disabled="!item.enabled"
          :placeholder="t('keyValue.value')"
          @update:model-value="handleValueChange(index, $event)"
        />

        <div class="flex items-center justify-center h-6">
          <select
            :value="item.type || 'text'"
            @change="handleTypeChange(index, ($event.target as HTMLSelectElement).value as 'text' | 'file')"
            class="w-full h-6 text-[11px] bg-transparent focus:outline-none text-text-primary cursor-pointer"
            :class="{ 'opacity-50': !item.enabled }"
          >
            <option value="text">{{ t('keyValue.text') }}</option>
            <option value="file">{{ t('keyValue.file') }}</option>
          </select>
        </div>

        <div class="flex items-center justify-center h-6">
          <button
            type="button"
            @click="handleDelete(index)"
            class="p-1 text-text-muted hover:text-error hover:bg-error/10 opacity-0 group-hover:opacity-100 transition-all duration-150 focus:outline-none"
            :title="t('keyValue.delete')"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Add Button -->
    <button
      type="button"
      @click="handleAdd"
      class="inline-flex items-center gap-1 px-2 py-1 text-xs text-accent hover:text-cyan-300 hover:bg-surface-elevated rounded transition-colors duration-150 font-['IBM_Plex_Sans']"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      {{ t('keyValue.add') }}
    </button>
  </div>
</template>
