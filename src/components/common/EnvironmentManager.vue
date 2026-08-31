<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useEnvironmentsStore, type Environment } from '../../stores/environments'
import { confirmDelete } from '../../composables/useConfirm'
import { toast } from '../../composables/useToast'
import VariableTable from './VariableTable.vue'

const { t } = useI18n()

const emit = defineEmits<{
  close: []
}>()

const environmentsStore = useEnvironmentsStore()

const selectedEnvId = ref<string | null>(null)
const searchQuery = ref('')
const newEnvName = ref('')
const showNewEnvInput = ref(false)
const editingEnvId = ref<string | null>(null)
const titleInputRef = ref<HTMLInputElement | null>(null)
const newEnvInputRef = ref<HTMLInputElement | null>(null)

// Context menu state
const contextMenu = ref({
  show: false,
  x: 0,
  y: 0,
  envId: null as string | null,
})

const GLOBAL_ENV_ID = '__global__'

onMounted(async () => {
  await environmentsStore.fetchEnvironments()
  // Select first environment by default
  const first = filteredEnvironments.value[0]
  if (first) {
    selectedEnvId.value = first.id
  }
  document.addEventListener('click', closeContextMenu)
  document.addEventListener('keydown', handleDocumentKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', closeContextMenu)
  document.removeEventListener('keydown', handleDocumentKeydown)
})

// Autofocus attributes are unreliable for dynamically inserted inputs,
// so focus explicitly when the rename / new-env inputs appear
watch(editingEnvId, async (id) => {
  if (id) {
    await nextTick()
    titleInputRef.value?.focus()
    titleInputRef.value?.select()
  }
})

watch(showNewEnvInput, async (visible) => {
  if (visible) {
    await nextTick()
    newEnvInputRef.value?.focus()
  }
})

const selectedEnv = computed(() => {
  if (!selectedEnvId.value) return null
  return environmentsStore.environments.find(e => e.id === selectedEnvId.value) || null
})

const filteredEnvironments = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  return environmentsStore.environments
    .filter(e => e.id !== GLOBAL_ENV_ID)
    .filter(e => !q || e.name.toLowerCase().includes(q))
    .sort((a, b) => a.name.localeCompare(b.name))
})

function selectEnv(id: string) {
  selectedEnvId.value = id
}

async function createNewEnv() {
  if (!newEnvName.value.trim()) return
  try {
    const created = await environmentsStore.createEnvironment({
      name: newEnvName.value.trim(),
      variables: {}
    })
    newEnvName.value = ''
    showNewEnvInput.value = false
    selectedEnvId.value = created.id
  } catch (e) {
    toast.error(t('environment.createFailed'))
  }
}

async function deleteEnv(id: string) {
  if (id === GLOBAL_ENV_ID) return
  if (!await confirmDelete(t('environment.deleteConfirm'))) return
  try {
    await environmentsStore.deleteEnvironment(id)
    if (selectedEnvId.value === id) {
      selectedEnvId.value = filteredEnvironments.value[0]?.id ?? null
    }
  } catch (e) {
    toast.error(t('environment.deleteFailed'))
  }
}

function uniqueDuplicateName(base: string): string {
  const names = new Set(environmentsStore.environments.map(e => e.name))
  if (!names.has(base)) return base
  let i = 2
  while (names.has(`${base} (${i})`)) i++
  return `${base} (${i})`
}

async function duplicateEnv(env: Environment) {
  if (env.id === GLOBAL_ENV_ID) return
  try {
    const created = await environmentsStore.createEnvironment({
      name: uniqueDuplicateName(t('environment.copyOf', { name: env.name })),
      variables: { ...env.variables }
    })
    selectedEnvId.value = created.id
  } catch (e) {
    toast.error(t('environment.createFailed'))
  }
}

async function updateEnvName(env: Environment, newName: string) {
  const name = newName.trim()
  if (!name || env.id === GLOBAL_ENV_ID || name === env.name) {
    editingEnvId.value = null
    return
  }
  try {
    await environmentsStore.updateEnvironment({
      id: env.id,
      name
    })
  } catch (e) {
    toast.error(t('environment.updateFailed'))
    // Keep the edit state so the typed name is not lost
    return
  }
  editingEnvId.value = null
}

function handleEnvContextMenu(event: MouseEvent, envId: string) {
  event.preventDefault()
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    envId,
  }
}

function closeContextMenu() {
  contextMenu.value.show = false
}

function handleRename() {
  if (contextMenu.value.envId) {
    selectedEnvId.value = contextMenu.value.envId
    editingEnvId.value = contextMenu.value.envId
  }
  closeContextMenu()
}

function handleDelete() {
  if (contextMenu.value.envId) {
    deleteEnv(contextMenu.value.envId)
  }
  closeContextMenu()
}

// Generic variable handlers
async function handleAddVariable(envId: string, key: string, value: string) {
  if (!key.trim()) return
  const env = environmentsStore.environments.find(e => e.id === envId)
  if (!env) return
  const newVars = { ...env.variables, [key.trim()]: value }
  try {
    await environmentsStore.updateEnvironment({ id: envId, variables: newVars })
  } catch (e) {
    toast.error(t('environment.updateFailed'))
  }
}

async function deleteVariable(envId: string, key: string) {
  try {
    await environmentsStore.deleteEnvironmentVariable(envId, key)
  } catch (e) {
    toast.error(t('environment.updateFailed'))
  }
}

async function updateVariable(envId: string, oldKey: string, newKey: string, value: string) {
  const env = environmentsStore.environments.find(e => e.id === envId)
  if (!env) return

  try {
    if (oldKey === 'REORDER') {
      const newOrder: Record<string, string>[] = JSON.parse(value)
      const reordered: Record<string, string> = {}
      for (const item of newOrder) {
        reordered[item.key] = item.value
      }
      await environmentsStore.updateEnvironment({ id: envId, variables: reordered })
      return
    }

    const newVars = { ...env.variables }
    if (oldKey !== newKey) {
      delete newVars[oldKey]
    }
    newVars[newKey] = value
    await environmentsStore.updateEnvironment({ id: envId, variables: newVars })
  } catch (e) {
    toast.error(t('environment.updateFailed'))
  }
}

function handleOverlayClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    emit('close')
  }
}

// Esc closes the modal, unless an input is focused (let its own Esc handler act)
function handleDocumentKeydown(e: KeyboardEvent) {
  if (e.key !== 'Escape') return
  const target = e.target as HTMLElement | null
  if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) return
  if (contextMenu.value.show) {
    closeContextMenu()
    return
  }
  emit('close')
}
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
    @click="handleOverlayClick"
  >
    <div class="bg-surface-base border border-border-default rounded w-full max-w-3xl mx-4 h-[85vh] flex flex-col overflow-hidden shadow-2xl">
      <!-- Header -->
      <div class="h-9 px-3 border-b border-border-default flex items-center justify-between flex-shrink-0 bg-surface-deep">
        <h2 class="text-[13px] font-semibold text-text-primary">{{ t('environment.manageEnvironments') }}</h2>
        <div class="flex items-center gap-0.5">
          <button
            @click="showNewEnvInput = true"
            class="p-1 hover:bg-surface-elevated rounded text-text-muted hover:text-text-primary transition-colors"
            :title="t('environment.newEnvironment')"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
          <button
            @click="emit('close')"
            class="p-1 hover:bg-surface-elevated rounded text-text-muted hover:text-text-primary transition-colors"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Body: sidebar + editor -->
      <div class="flex flex-1 overflow-hidden">
        <!-- Left: environment list -->
        <div class="w-52 border-r border-border-default flex flex-col bg-surface-deep flex-shrink-0">
          <!-- Search -->
          <div class="m-2 h-[26px] flex items-center gap-1.5 bg-input-bg border border-border-default focus-within:border-border-focus rounded px-2 text-text-muted">
            <svg class="w-3 h-3 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-4.35-4.35M17 11a6 6 0 11-12 0 6 6 0 0112 0z" />
            </svg>
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="t('environment.searchPlaceholder')"
              class="flex-1 min-w-0 bg-transparent border-0 outline-none text-xs text-text-primary placeholder-text-muted"
              @keyup.escape="searchQuery = ''"
            />
          </div>

          <!-- List -->
          <div class="flex-1 overflow-auto pb-1">
            <div class="px-3 pb-1 text-[11px] font-semibold text-text-muted">
              {{ t('environment.allEnvironments') }}
              <span class="font-normal opacity-70">{{ filteredEnvironments.length }}</span>
            </div>

            <div
              v-for="env in filteredEnvironments"
              :key="env.id"
              @click="selectEnv(env.id)"
              @contextmenu="handleEnvContextMenu($event, env.id)"
              class="flex items-center gap-2 h-[26px] px-3 text-[13px] cursor-pointer"
              :class="selectedEnvId === env.id
                ? 'bg-list-active text-list-active-fg'
                : 'text-text-primary hover:bg-list-hover'"
            >
              <svg
                class="w-3.5 h-3.5 flex-shrink-0 opacity-70"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"
              >
                <path d="M12 3l9 5-9 5-9-5 9-5z" />
                <path stroke-linecap="round" d="M3 13.5l9 5 9-5" />
              </svg>
              <span class="truncate">{{ env.name }}</span>
            </div>

            <!-- New Environment inline input -->
            <div v-if="showNewEnvInput" class="px-2 py-1">
              <input
                ref="newEnvInputRef"
                v-model="newEnvName"
                @keyup.enter="createNewEnv"
                @keyup.escape="showNewEnvInput = false"
                type="text"
                :placeholder="t('environment.name')"
                class="w-full h-6 px-2 text-xs bg-input-bg border border-border-focus rounded outline-none text-text-primary placeholder-text-muted"
              />
              <div class="flex gap-1 mt-1">
                <button
                  @click="createNewEnv"
                  class="flex-1 px-2 py-0.5 text-[11px] bg-accent text-white hover:bg-accent/80 rounded"
                >
                  {{ t('environment.create') }}
                </button>
                <button
                  @click="showNewEnvInput = false"
                  class="px-2 py-0.5 text-[11px] text-text-muted hover:text-text-primary"
                >
                  {{ t('environment.cancel') }}
                </button>
              </div>
            </div>

            <!-- Globals pinned -->
            <div class="h-px bg-border-default my-1.5"></div>
            <div
              @click="selectEnv(GLOBAL_ENV_ID)"
              class="flex items-center gap-2 h-[26px] px-3 text-[13px] cursor-pointer"
              :class="selectedEnvId === GLOBAL_ENV_ID
                ? 'bg-list-active text-list-active-fg'
                : 'text-text-primary hover:bg-list-hover'"
            >
              <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-70" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
                <path d="M12 2a10 10 0 100 20 10 10 0 000-20zm0 0c3 3.5 3 17.5 0 20M2 12h20" />
              </svg>
              <span class="truncate">{{ t('environment.globalVariables') }}</span>
            </div>
          </div>

          <!-- Add Button -->
          <button
            v-if="!showNewEnvInput"
            @click="showNewEnvInput = true"
            class="mx-2 mb-2 px-2 py-1.5 text-xs text-text-secondary hover:text-accent flex items-center gap-1.5 border border-dashed border-border-default hover:border-accent rounded transition-colors"
          >
            <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            {{ t('environment.newEnvironment') }}
          </button>
        </div>

        <!-- Right: variables of the selected environment -->
        <div class="flex-1 flex flex-col overflow-hidden bg-surface-base min-w-0">
          <template v-if="selectedEnv">
            <!-- Title bar -->
            <div class="group h-9 px-3.5 border-b border-border-default flex items-center gap-2 flex-shrink-0">
              <input
                v-if="editingEnvId === selectedEnv.id"
                ref="titleInputRef"
                :value="selectedEnv.name"
                @keyup.enter="updateEnvName(selectedEnv, ($event.target as HTMLInputElement).value)"
                @blur="updateEnvName(selectedEnv, ($event.target as HTMLInputElement).value)"
                @keyup.escape="editingEnvId = null"
                class="h-6 w-56 px-2 text-[13px] bg-input-bg border border-border-focus rounded outline-none text-text-primary"
              />
              <template v-else>
                <span class="text-[13px] font-semibold text-text-primary truncate">
                  {{ selectedEnv.id === GLOBAL_ENV_ID ? t('environment.globalVariables') : selectedEnv.name }}
                </span>
                <span class="text-[11px] text-text-muted flex-shrink-0">
                  {{ t('environment.varCount', { count: Object.keys(selectedEnv.variables).length }) }}
                </span>
              </template>

              <div
                v-if="selectedEnv.id !== GLOBAL_ENV_ID"
                class="ml-auto flex items-center gap-0.5 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity"
              >
                <button
                  @click="editingEnvId = selectedEnv.id"
                  class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-surface-elevated transition-colors"
                  :title="t('environment.rename')"
                >
                  <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button
                  @click="duplicateEnv(selectedEnv)"
                  class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-surface-elevated transition-colors"
                  :title="t('environment.duplicate')"
                >
                  <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 8h12v12H8zM4 16V4h12" />
                  </svg>
                </button>
                <button
                  @click="deleteEnv(selectedEnv.id)"
                  class="p-1 rounded text-text-muted hover:text-error hover:bg-error/10 transition-colors"
                  :title="t('environment.delete')"
                >
                  <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>

            <!-- Variables table -->
            <div class="flex-1 overflow-auto">
              <VariableTable
                :variables="selectedEnv.variables"
                :env-id="selectedEnv.id"
                @add-variable="(key, value) => handleAddVariable(selectedEnv!.id, key, value)"
                @delete-variable="(key) => deleteVariable(selectedEnv!.id, key)"
                @update-variable="(oldKey, newKey, value) => updateVariable(selectedEnv!.id, oldKey, newKey, value)"
              />
            </div>
          </template>

          <div v-else class="flex-1 flex items-center justify-center text-text-muted">
            <div class="text-center">
              <svg class="w-10 h-10 mx-auto text-text-muted mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              <div class="text-sm font-medium text-text-primary">{{ t('environment.selectOne') }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Context Menu -->
      <div
        v-if="contextMenu.show"
        class="fixed z-[100] bg-surface-base shadow-xl border border-border-default py-1 min-w-[120px] rounded"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      >
        <button
          class="w-full px-3 py-1.5 text-xs text-left text-text-secondary hover:bg-surface-elevated"
          @click="handleRename"
        >
          {{ t('environment.rename') }}
        </button>
        <button
          class="w-full px-3 py-1.5 text-xs text-left text-error hover:bg-surface-elevated"
          @click="handleDelete"
        >
          {{ t('environment.delete') }}
        </button>
      </div>
    </div>
  </div>
</template>
