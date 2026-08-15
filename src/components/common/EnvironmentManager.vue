<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
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
const newEnvName = ref('')
const showNewEnvInput = ref(false)
const editingEnvId = ref<string | null>(null)

// Context menu state
const contextMenu = ref({
  show: false,
  x: 0,
  y: 0,
  envId: null as string | null,
})

// Split bar state - applies to whole modal
const splitRatio = ref(0.5)
const isDragging = ref(false)
const containerRef = ref<HTMLElement | null>(null)
const startDragY = ref(0)
const startRatio = ref(0.5)

const GLOBAL_ENV_ID = '__global__'

const globalEnv = computed(() => {
  return environmentsStore.environments.find(e => e.id === GLOBAL_ENV_ID) || null
})

onMounted(async () => {
  await environmentsStore.fetchEnvironments()
  // Select first environment by default
  const first = filteredEnvironments.value[0]
  if (first) {
    selectedEnvId.value = first.id
  }
  document.addEventListener('click', closeContextMenu)
})

onUnmounted(() => {
  document.removeEventListener('click', closeContextMenu)
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
})

const selectedEnv = computed(() => {
  if (!selectedEnvId.value) return null
  return environmentsStore.environments.find(e => e.id === selectedEnvId.value) || null
})

const filteredEnvironments = computed(() => {
  return environmentsStore.environments
    .filter(e => e.id !== GLOBAL_ENV_ID)
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
      selectedEnvId.value = null
    }
  } catch (e) {
    toast.error(t('environment.deleteFailed'))
  }
}

async function updateEnvName(env: Environment, newName: string) {
  if (!newName.trim() || env.id === GLOBAL_ENV_ID) return
  try {
    await environmentsStore.updateEnvironment({
      id: env.id,
      name: newName.trim()
    })
  } catch (e) {
    toast.error(t('environment.updateFailed'))
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

// Resize handlers - for whole modal split
function startDrag(event: MouseEvent) {
  event.preventDefault()
  isDragging.value = true
  startDragY.value = event.clientY
  startRatio.value = splitRatio.value
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

function onDrag(event: MouseEvent) {
  if (!isDragging.value || !containerRef.value) return
  event.preventDefault()
  const rect = containerRef.value.getBoundingClientRect()
  const delta = event.clientY - startDragY.value
  const deltaRatio = delta / rect.height
  splitRatio.value = Math.min(0.75, Math.max(0.25, startRatio.value + deltaRatio))
}

function stopDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

function handleOverlayClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    emit('close')
  }
}
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    @click="handleOverlayClick"
  >
    <div ref="containerRef" class="bg-surface-base rounded-lg shadow-xl w-full max-w-3xl mx-4 h-[85vh] flex flex-col overflow-hidden">
      <!-- Header -->
      <div class="px-4 py-2 border-b border-border-default flex items-center justify-between flex-shrink-0 bg-surface-deep">
        <h2 class="text-sm font-medium text-text-primary">{{ t('environment.manageEnvironments') }}</h2>
        <button
          @click="emit('close')"
          class="p-1 hover:bg-surface-elevated rounded"
        >
          <svg class="w-4 h-4 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Top: Environment Selection + Selected Env Variables -->
      <div
        class="flex overflow-hidden"
        :style="{ height: `${splitRatio * 100}%` }"
      >
        <!-- Left Sidebar: Environment List -->
        <div class="w-48 border-r border-border-default flex flex-col bg-surface-deep">
          <!-- Environment List -->
          <div class="flex-1 overflow-auto">
            <div
              v-for="env in filteredEnvironments"
              :key="env.id"
              @click="selectEnv(env.id)"
              @contextmenu="handleEnvContextMenu($event, env.id)"
              class="group px-3 py-1.5 cursor-pointer transition-colors border-l-2"
              :class="selectedEnvId === env.id ? 'bg-surface-elevated border-l-transparent' : 'border-l-transparent bg-surface-deep'"
            >
              <input
                v-if="editingEnvId === env.id"
                :value="env.name"
                @keyup.enter="updateEnvName(env, ($event.target as HTMLInputElement).value)"
                @blur="editingEnvId = null"
                @keyup.escape="editingEnvId = null"
                @click.stop
                class="w-full px-1 py-0.5 text-xs border border-accent bg-surface-deep text-text-primary"
                autofocus
              />
              <span v-else class="text-sm font-medium text-text-primary truncate block">
                {{ env.name }}
              </span>
            </div>

            <!-- New Environment Input -->
            <div v-if="showNewEnvInput" class="px-2 py-1.5 border-t border-border-default bg-surface-base">
              <input
                v-model="newEnvName"
                @keyup.enter="createNewEnv"
                @keyup.escape="showNewEnvInput = false"
                type="text"
                :placeholder="t('environment.name')"
                class="w-full px-2 py-1 text-xs border border-accent bg-surface-deep text-text-primary"
                autofocus
              />
              <div class="flex gap-1 mt-1.5">
                <button
                  @click="createNewEnv"
                  class="flex-1 px-2 py-0.5 text-[10px] bg-accent text-white hover:bg-accent/80"
                >
                  {{ t('environment.create') }}
                </button>
                <button
                  @click="showNewEnvInput = false"
                  class="px-2 py-0.5 text-[10px] text-text-muted"
                >
                  {{ t('environment.cancel') }}
                </button>
              </div>
            </div>
          </div>

          <!-- Add Button -->
          <button
            v-if="!showNewEnvInput"
            @click="showNewEnvInput = true"
            class="w-full px-3 py-1.5 text-xs text-accent hover:bg-surface-base flex items-center gap-1.5 border-t border-border-default"
          >
            <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            {{ t('environment.newEnvironment') }}
          </button>
        </div>

        <!-- Right: Selected Environment Variables -->
        <div class="flex-1 flex flex-col overflow-hidden bg-surface-base">
          <template v-if="selectedEnv">
            <VariableTable
              :variables="selectedEnv.variables"
              :env-id="selectedEnv.id"
              @add-variable="(key, value) => handleAddVariable(selectedEnv!.id, key, value)"
              @delete-variable="(key) => deleteVariable(selectedEnv!.id, key)"
              @update-variable="(oldKey, newKey, value) => updateVariable(selectedEnv!.id, oldKey, newKey, value)"
            />
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

      <!-- Resizable Split Bar -->
      <div
        class="relative z-10 select-none cursor-row-resize"
        style="height: 1px; background: transparent"
        @mousedown="startDrag"
      >
        <!-- Visual line (very thin) -->
        <div
          class="absolute left-0 right-0 top-0 h-px transition-all duration-150"
          :style="
            isDragging
              ? 'background-color: rgba(0, 122, 204, 0.5); height: 2px'
              : 'background-color: var(--border-default)'
          "
        />

        <!-- Hover zone (transparent 20px tall area) -->
        <div
          class="absolute -top-2 -bottom-2 left-0 right-0"
          :class="{ 'cursor-row-resize': isDragging }"
        />
      </div>

      <!-- Bottom: Global Variables (full width) -->
      <div
        class="flex flex-col overflow-hidden"
        :style="{ height: `${(1 - splitRatio) * 100}%` }"
      >
        <template v-if="globalEnv">
          <div class="px-4 py-2 border-b border-border-default bg-surface-deep flex-shrink-0">
            <h3 class="text-sm font-medium text-text-primary">
              {{ t('environment.globalVariables') }}
            </h3>
          </div>

          <VariableTable
            :variables="globalEnv.variables"
            :env-id="globalEnv.id"
            @add-variable="(key, value) => handleAddVariable(GLOBAL_ENV_ID, key, value)"
            @delete-variable="(key) => deleteVariable(GLOBAL_ENV_ID, key)"
            @update-variable="(oldKey, newKey, value) => updateVariable(GLOBAL_ENV_ID, oldKey, newKey, value)"
          />
        </template>
      </div>

      <!-- Context Menu -->
      <div
        v-if="contextMenu.show"
        class="fixed z-[100] bg-surface-base shadow-xl border border-border-default py-1 min-w-[120px]"
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
