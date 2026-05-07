<script setup lang="ts">
import KeyValueEditor from './KeyValueEditor.vue'
import type { KeyValuePair } from '../../stores/types'

defineProps<{
  modelValue: KeyValuePair[]
  variables?: Record<string, string>
}>()

const emit = defineEmits<{
  'update:modelValue': [items: KeyValuePair[]]
}>()

function handleUpdate(items: KeyValuePair[]) {
  emit('update:modelValue', items)
}
</script>

<template>
  <div class="p-2">
    <KeyValueEditor :model-value="modelValue" :variables="variables" @update:model-value="handleUpdate" />
  </div>
</template>
