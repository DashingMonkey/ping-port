<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

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

// New variable state
const isAddingNew = ref(false)
const newKey = ref('')
const newValue = ref('')

// Handle key change - save on blur
function handleKeyChange(index: number, newKeyValue: string) {
  const item = filteredVariables.value[index]
  if (newKeyValue.trim() !== item.key) {
    emit('updateVariable', item.key, newKeyValue.trim(), item.value)
  }
}

// Handle value change - save on blur
function handleValueChange(index: number, newValue: string) {
  const item = filteredVariables.value[index]
  if (newValue !== item.value) {
    emit('updateVariable', item.key, item.key, newValue)
  }
}

// Start adding new
function startAddNew() {
  isAddingNew.value = true
  newKey.value = ''
  newValue.value = ''
}

// Save new variable
function saveNew() {
  const key = newKey.value.trim()
  const value = newValue.value
  if (key) {
    emit('addVariable', key, value)
  }
  cancelAdd()
}

function cancelAdd() {
  isAddingNew.value = false
  newKey.value = ''
  newValue.value = ''
}
</script>

<template>
  <div class="space-y-0">
    <!-- Table wrapper -->
    <div class="border-b border-border-default">
      <!-- Header Row -->
      <div class="grid grid-cols-[1fr_1fr_40px] divide-x divide-border-default bg-surface-base">
        <div class="px-3 flex items-center h-6">
          <span class="text-[10px] font-semibold tracking-wide text-text-muted uppercase">{{ t('keyValue.name') }}</span>
        </div>
        <div class="px-3 flex items-center h-6">
          <span class="text-[10px] font-semibold tracking-wide text-text-muted uppercase">{{ t('keyValue.value') }}</span>
        </div>
        <div class="flex items-center justify-center h-6">
          <span class="w-4"></span>
        </div>
      </div>

      <!-- Data Rows -->
      <div
        v-for="(item, index) in filteredVariables"
        :key="item.key"
        class="grid grid-cols-[1fr_1fr_40px] divide-x divide-border-default border-t border-border-default group"
      >
        <!-- Key column -->
        <input
          type="text"
          :value="item.key"
          @change="handleKeyChange(index, ($event.target as HTMLInputElement).value)"
          :placeholder="t('keyValue.name')"
          class="w-full h-6 px-3 text-[10px] border-0 bg-transparent focus:outline-none text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
        />

        <!-- Value column -->
        <input
          type="text"
          :value="item.value"
          @change="handleValueChange(index, ($event.target as HTMLInputElement).value)"
          :placeholder="t('keyValue.value')"
          class="w-full h-6 px-3 text-[10px] border-0 bg-transparent focus:outline-none text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
        />

        <!-- Delete column -->
        <div class="flex items-center justify-center h-6">
          <button
            type="button"
            @click="emit('deleteVariable', item.key)"
            class="p-1 text-text-muted hover:text-error hover:bg-error/10 opacity-30 hover:opacity-100 transition-opacity duration-150 cursor-pointer"
            :title="t('keyValue.delete')"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </div>

      <!-- New variable row -->
      <div
        v-if="isAddingNew"
        class="grid grid-cols-[1fr_1fr_40px] divide-x divide-border-default border-t border-border-default"
      >
        <input
          v-model="newKey"
          type="text"
          :placeholder="t('keyValue.name')"
          class="w-full h-6 px-3 text-[10px] border-0 bg-surface-deep focus:outline-none text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
          @keyup.enter="saveNew"
          @keyup.escape="cancelAdd"
          autofocus
        />
        <input
          v-model="newValue"
          type="text"
          :placeholder="t('keyValue.value')"
          class="w-full h-6 px-3 text-[10px] border-0 bg-surface-deep focus:outline-none text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
          @keyup.enter="saveNew"
          @keyup.escape="cancelAdd"
        />
        <div class="flex items-center justify-center h-6">
          <button
            @click="saveNew"
            class="p-1 text-success hover:bg-success/10 transition-colors"
            :title="t('keyValue.save')"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Add Button -->
    <button
      v-if="!isAddingNew"
      type="button"
      @click="startAddNew"
      class="inline-flex items-center gap-1 px-2 py-1 text-xs text-accent hover:text-cyan-300 hover:bg-surface-elevated transition-colors duration-150 font-['IBM_Plex_Sans']"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      {{ t('keyValue.addVariable') }}
    </button>

    <!-- Empty state -->
    <div
      v-if="filteredVariables.length === 0 && !isAddingNew"
      class="p-8 text-center text-text-muted"
    >
      <svg class="w-10 h-10 mx-auto text-text-muted/50 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <div class="text-sm font-medium text-text-primary">{{ t('keyValue.noVariables') }}</div>
    </div>
  </div>
</template>
