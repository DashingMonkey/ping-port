<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from '@headlessui/vue'
import MethodSelect from './MethodSelect.vue'

const { t } = useI18n()

type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'OPTIONS' | 'HEAD'

export interface Collection {
  id: string
  name: string
}

const props = defineProps<{
  modelValue: string
  method: HttpMethod
  title: string
  collectionId?: string
  collections?: Collection[]
}>()

const emit = defineEmits<{
  'update:modelValue': [url: string]
  'update:method': [method: HttpMethod]
  'update:title': [title: string]
  'update:collectionId': [collectionId: string]
  'send': []
  'save': []
}>()

const selectedCollectionName = computed(() => {
  if (!props.collectionId || !props.collections) return ''
  const found = props.collections.find(c => c.id === props.collectionId)
  return found ? found.name : ''
})

// Title editing state
const isEditingTitle = ref(false)
const editedTitle = ref('')

function handleUrlInput(event: Event) {
  const target = event.target as HTMLTextAreaElement
  if (target.value.endsWith('?') || target.value.endsWith('？')) {
    target.value = target.value.slice(0, -1)
  }
  emit('update:modelValue', target.value)
}

function handleTitleClick() {
  isEditingTitle.value = true
  editedTitle.value = props.title
}

function handleTitleEdit(event: Event) {
  const target = event.target as HTMLInputElement
  editedTitle.value = target.value
}

function handleTitleBlur() {
  if (editedTitle.value.trim()) {
    emit('update:title', editedTitle.value.trim())
  }
  isEditingTitle.value = false
}

function handleTitleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    if (editedTitle.value.trim()) {
      emit('update:title', editedTitle.value.trim())
    }
    isEditingTitle.value = false
  } else if (event.key === 'Escape') {
    isEditingTitle.value = false
  }
}

function handleMethodUpdate(method: HttpMethod) {
  emit('update:method', method)
}

function handleCollectionChange(collectionId: string) {
  emit('update:collectionId', collectionId)
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    emit('send')
  }
}

function handleSend() {
  emit('send')
}

function handleSave() {
  emit('save')
}
</script>

<template>
  <div class="space-y-1">
    <!-- Title Row -->
    <div class="flex items-center gap-1 bg-surface-base">
      <!-- Breadcrumb with collection dropdown -->
      <div class="flex items-center gap-1 flex-1 min-w-0">
        <Listbox
          :model-value="collectionId"
          @update:model-value="handleCollectionChange"
        >
          <div class="relative">
            <ListboxButton
              class="text-xs bg-transparent border-none text-text-muted focus:outline-none cursor-pointer py-0.5 pr-1 font-['IBM_Plex_Sans'] flex items-center gap-0.5"
            >
              <span class="truncate">{{ selectedCollectionName || t('request.selectCollection') }}</span>
              <svg class="w-3 h-3 opacity-60 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
              </svg>
            </ListboxButton>

            <transition
              leave-active-class="transition duration-100 ease-in"
              leave-from-class="opacity-100"
              leave-to-class="opacity-0"
            >
              <ListboxOptions
                class="absolute z-10 mt-1 bg-surface-base shadow-xl max-h-60 rounded-md border border-border-default focus:outline-none overflow-hidden w-max"
              >
                <ListboxOption
                  v-for="c in collections"
                  :key="c.id"
                  :value="c.id"
                  v-slot="{ active, selected }"
                >
                  <li
                    class="cursor-pointer select-none relative py-1.5 pl-3 pr-9 transition-colors duration-100 text-xs whitespace-nowrap"
                    :class="[
                      active ? 'bg-accent/10 text-text-primary' : 'text-text-secondary',
                      selected ? 'font-semibold' : 'font-normal'
                    ]"
                  >
                    {{ c.name }}
                    <span v-if="selected" class="absolute inset-y-0 right-0 flex items-center pr-3 text-accent">
                      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      </svg>
                    </span>
                  </li>
                </ListboxOption>
              </ListboxOptions>
            </transition>
          </div>
        </Listbox>
        <span class="text-xs text-text-muted">/</span>
        <template v-if="!isEditingTitle">
          <span
            @click="handleTitleClick"
            class="text-xs text-text-primary truncate font-['IBM_Plex_Sans'] cursor-pointer hover:text-accent transition-colors"
          >
            {{ title || t('tabs.newRequest') }}
          </span>
        </template>
        <template v-else>
          <input
            ref="titleInputRef"
            type="text"
            :value="editedTitle"
            @input="handleTitleEdit"
            @blur="handleTitleBlur"
            @keydown="handleTitleKeydown"
            class="min-w-[80px] max-w-[300px] block rounded border border-border-accent bg-surface-base px-2 py-0.5 text-xs focus:outline-none text-text-primary placeholder-text-muted font-['IBM_Plex_Sans']"
            :placeholder="t('request.requestNamePlaceholder')"
            autofocus
          />
        </template>
      </div>

      <button
        type="button"
        @click="handleSave"
        :disabled="!title.trim() || !collectionId"
        class="inline-flex items-center justify-center w-[60px] px-2 py-1.5 border border-transparent rounded bg-surface-base text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-surface-elevated hover:border-border-default focus:outline-none transition-colors duration-150 font-['IBM_Plex_Sans'] disabled:opacity-50 disabled:cursor-not-allowed"
        :title="t('request.saveTooltip')"
      >
        <svg class="w-3 h-3 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
        </svg>
        {{ t('request.save') }}
      </button>
    </div>

    <!-- URL Row -->
    <div class="flex items-center gap-1 bg-surface-base">
      <MethodSelect
        :model-value="method"
        @update:model-value="handleMethodUpdate"
      />

      <input
        type="text"
        :value="modelValue"
        @input="handleUrlInput"
        @keydown="handleKeydown"
        :placeholder="t('request.urlPlaceholder')"
        class="flex-1 min-w-0 block rounded-r border border-border-default bg-surface-base px-2 py-1.5 text-xs focus:outline-none focus:border-accent text-text-primary placeholder-text-muted font-['JetBrains_Mono'] text-[11px]"
      />

      <button
        type="button"
        @click="handleSend"
        class="inline-flex items-center justify-center w-[60px] px-2 py-1.5 border border-border-default rounded bg-accent hover:bg-cyan-400 text-white text-xs font-semibold focus:outline-none focus:ring-2 focus:ring-accent/50 transition-colors duration-150 font-['IBM_Plex_Sans']"
      >
        <svg class="w-3 h-3 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
        </svg>
        {{ t('request.send') }}
      </button>
    </div>
  </div>
</template>
