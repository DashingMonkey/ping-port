<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import AppLayout from './components/layout/AppLayout.vue'
import EmptyState from './components/common/EmptyState.vue'
import RequestPanel from './components/request/RequestPanel.vue'
import { useI18n } from 'vue-i18n'
import { useTabsStore } from './stores/tabs'
import { useCollectionsStore } from './stores/collections'
import { useRequestsStore } from './stores/requests'
import { useSettingsStore } from './stores/settings'
import { useWorkspaceStore } from './stores/workspace'
import type { RequestState } from './stores/types'

interface RustWorkspace {
  name: string
  path: string
  is_temporary: boolean
}

interface InitWorkspaceResult {
  current_workspace: string
  workspaces: RustWorkspace[]
}

const tabsStore = useTabsStore()
const requestsStore = useRequestsStore()
const collectionsStore = useCollectionsStore()
const settingsStore = useSettingsStore()
const workspaceStore = useWorkspaceStore()
const { t } = useI18n()

const isInitializing = ref(true)
const initError = ref<string | null>(null)
const isRetrying = ref(false)
// Bumped when an external writer (MCP) changes the open request without a
// local draft, so the panel remounts with fresh data.
const externalRequestVersion = ref(0)

async function initializeApp() {
  isInitializing.value = true
  initError.value = null

  try {
    // 1. Initialize workspace (backend scans, decides which db to use, registers AppState)
    const lastWorkspaceName = localStorage.getItem('pingport-last-workspace')
    const result = await invoke<InitWorkspaceResult>('init_workspace', {
      lastWorkspaceName: lastWorkspaceName || null
    })

    // Set current workspace and workspaces list
    workspaceStore.currentWorkspace = {
      name: result.current_workspace,
      path: '',
      isTemporary: result.current_workspace.startsWith('temp-')
    }
    workspaceStore.workspaces = result.workspaces.map(w => ({
      name: w.name,
      path: w.path,
      isTemporary: w.is_temporary,
    }))
    localStorage.setItem('pingport-last-workspace', result.current_workspace)

    // Only scan if workspaces is empty (first run created default.db)
    if (result.workspaces.length === 0) {
      await workspaceStore.scanWorkspaces()
    }

    // 2. Load data for the current workspace
    await workspaceStore.refreshStoresForWorkspace()

    // 3. Continue normal initialization
    finishInit()
  } catch (e: unknown) {
    const msg = typeof e === 'string' ? e : (e instanceof Error ? e.message : String(e))
    initError.value = msg
    isInitializing.value = false
  }
}

function retryInit() {
  isRetrying.value = true
  initError.value = null
  initializeApp().finally(() => {
    isRetrying.value = false
  })
}

function copyError() {
  if (initError.value) {
    navigator.clipboard.writeText(initError.value).catch(() => {})
  }
}

async function finishInit() {
  try {
    // Load settings (workspace is determined at this point)
    settingsStore.loadSettings()

    // Load tabs and drafts (from current workspace's localStorage)
    tabsStore.loadTabs()
  } finally {
    isInitializing.value = false
  }
}

onMounted(async () => {
  // Register external-sync listeners before initializing so MCP writes that
  // land during init are not missed.
  await setupExternalSync()
  initializeApp()
})

/** Close tabs whose saved request no longer exists (e.g. deleted via MCP). */
function closeOrphanTabs() {
  const known = new Set(requestsStore.requests.map(r => r.id))
  const orphanIds = tabsStore.tabs
    .filter(tab => tab.requestId && !known.has(tab.requestId))
    .map(tab => tab.id)
  orphanIds.forEach(id => tabsStore.closeTab(id))
}

/** React to data written by external writers (the MCP server). */
async function setupExternalSync() {
  await listen<{ source?: string; ids?: string[] }>('collections-changed', async () => {
    await collectionsStore.fetchCollections()
  })

  await listen<{ source?: string; ids?: string[] }>('requests-changed', async (event) => {
    await requestsStore.fetchRequests()
    closeOrphanTabs()

    const tab = tabsStore.activeTab
    const affected = event.payload?.ids
    if (tab?.requestId && affected?.includes(tab.requestId)) {
      // A draft exists for any opened request (created on panel mount), so
      // the guard must check for unsaved edits, not draft existence.
      const draft = tabsStore.drafts.find(d => d.tabId === tab.id)
      const hasEdits = (draft?.changesCount ?? 0) > 0
      if (!hasEdits) {
        // Drop the clean draft so the remounted panel reloads the updated
        // request instead of the stale draft state.
        if (draft) tabsStore.removeDraft(draft.id)
        externalRequestVersion.value++
      }
    }
  })
}

const initialState = computed<RequestState | undefined>(() => {
  const tab = tabsStore.activeTab
  if (!tab) return undefined

  // Check for draft first (applies to both new AND existing requests)
  const draft = tabsStore.drafts.find(d => d.tabId === tab.id)
  if (draft) {
    return draft.state
  }

  // No draft - load from saved request (for existing) or undefined (for new)
  if (!tab.requestId) {
    return undefined
  }

  const request = requestsStore.requests.find(r => r.id === tab.requestId)
  if (!request) return undefined
  return {
    method: request.method as RequestState['method'],
    url: request.url,
    params: parseJson(request.params, []),
    headers: parseJson(request.headers, []),
    body: parseJson(request.body, { type: 'none', content: '' }),
    auth: parseJson(request.auth, { type: 'none' }),
    preRequestScript: request.preRequestScript ?? '',
    testScript: request.testScript ?? '',
  }
})

const initialTitle = computed(() => {
  const tab = tabsStore.activeTab
  if (!tab) return ''
  if (!tab.requestId) return ''
  const request = requestsStore.requests.find(r => r.id === tab.requestId)
  return request?.name ?? ''
})

const requestId = computed(() => {
  const tab = tabsStore.activeTab
  if (!tab || !tab.requestId) return undefined
  return tab.requestId
})

function parseJson<T>(value: string | null, fallback: T): T {
  if (!value) return fallback
  try {
    return JSON.parse(value) as T
  } catch {
    return fallback
  }
}
</script>

<template>
  <AppLayout>
    <div v-if="initError" class="h-full flex items-center justify-center bg-[#1e1e1e]">
      <div class="text-center max-w-lg px-6">
        <div class="text-red-400 text-5xl mb-4">!</div>
        <h2 class="text-white text-lg font-semibold mb-2">{{ t('app.initFailed') }}</h2>
        <p class="text-gray-400 text-sm mb-4 whitespace-pre-wrap break-all max-h-40 overflow-auto bg-[#2d2d2d] rounded p-3 text-left font-mono text-xs border border-red-800/30">
          {{ initError }}
        </p>
        <p class="text-gray-500 text-xs mb-6">{{ t('app.initErrorHint') }}</p>
        <div class="flex gap-3 justify-center">
          <button
            class="px-5 py-2 bg-[#0078d4] hover:bg-[#1a8ad4] text-white text-sm rounded transition-colors disabled:opacity-50"
            :disabled="isRetrying"
            @click="retryInit"
          >
            {{ isRetrying ? t('app.retrying') : t('app.retry') }}
          </button>
          <button
            class="px-5 py-2 bg-[#3d3d3d] hover:bg-[#4d4d4d] text-gray-200 text-sm rounded transition-colors"
            @click="copyError"
          >
            {{ t('app.copyError') }}
          </button>
        </div>
        <p class="text-gray-600 text-xs mt-4">
          {{ t('app.webview2Hint') }}<br>
          <a href="https://go.microsoft.com/fwlink/p/?LinkId=2124703" class="text-blue-400 hover:text-blue-300 underline" target="_blank">https://go.microsoft.com/fwlink/p/?LinkId=2124703</a>
        </p>
      </div>
    </div>
    <div v-else-if="isInitializing" class="h-full flex items-center justify-center">
      <EmptyState
        :title="t('app.loading')"
        :message="t('app.loadingInit')"
      />
    </div>
    <div v-else-if="tabsStore.activeTab && initialState" class="h-full">
      <RequestPanel
        :key="`${tabsStore.activeTab.id}-${externalRequestVersion}`"
        :initial-state="initialState"
        :initial-title="initialTitle"
        :request-id="requestId"
      />
    </div>
    <div v-else-if="tabsStore.activeTab" class="h-full flex items-center justify-center">
      <EmptyState
        :title="t('app.loading')"
        :message="t('app.loadingRequest')"
      />
    </div>
    <div v-else class="h-full flex items-center justify-center">
      <EmptyState
        :title="t('app.noRequestSelected')"
        :message="t('app.noRequestMessage')"
      />
    </div>
  </AppLayout>
</template>
