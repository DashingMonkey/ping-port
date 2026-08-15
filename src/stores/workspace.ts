import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useCollectionsStore } from './collections'
import { useRequestsStore } from './requests'
import { useEnvironmentsStore } from './environments'
import { useTabsStore } from './tabs'
import { useSettingsStore } from './settings'

export interface Workspace {
  name: string
  path: string
  isTemporary: boolean
}

/** Raw workspace shape returned from Rust backend */
export interface RustWorkspace {
  name: string
  path: string
  is_temporary: boolean
}

interface ScanResult {
  workspaces: RustWorkspace[]
}

function transformWorkspace(raw: RustWorkspace): Workspace {
  return {
    name: raw.name,
    path: raw.path,
    isTemporary: raw.is_temporary,
  }
}

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([])
  const currentWorkspace = ref<Workspace | null>(null)
  const isLoading = ref(false)

  const error = ref<string | null>(null)

  // Read last used workspace from localStorage
  const lastWorkspaceName = ref<string | null>(localStorage.getItem('pingport-last-workspace'))

  // Scan available workspaces (just list, don't switch)
  const scanWorkspaces = async () => {
    isLoading.value = true
    try {
      const result = await invoke<ScanResult>('scan_workspaces')
      workspaces.value = result.workspaces.map(transformWorkspace)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // Switch workspace
  const switchWorkspace = async (name: string) => {
    isLoading.value = true
    try {
      await invoke('switch_workspace', { name })
      currentWorkspace.value = workspaces.value.find(w => w.name === name) || {
        name,
        path: '',
        isTemporary: name.startsWith('temp-')
      }
      localStorage.setItem('pingport-last-workspace', name)
      lastWorkspaceName.value = name

      // Trigger stores to reload
      await refreshStoresForWorkspace()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // Reload data for the current workspace
  const refreshStoresForWorkspace = async () => {
    const collectionsStore = useCollectionsStore()
    const requestsStore = useRequestsStore()
    const environmentsStore = useEnvironmentsStore()
    const tabsStore = useTabsStore()
    const settingsStore = useSettingsStore()

    await Promise.all([
      collectionsStore.fetchCollections(),
      requestsStore.fetchRequests(),
      environmentsStore.fetchEnvironments(),
      tabsStore.loadTabs(),
      settingsStore.loadSettings(),
    ])
  }

  // Create a temporary workspace
  const createTemporaryWorkspace = async () => {
    const tempName = `temp-${Date.now()}`
    await invoke('create_workspace', { name: tempName })
    await switchWorkspace(tempName)
  }

  // Rename a workspace
  const renameWorkspace = async (oldName: string, newName: string) => {
    await invoke('rename_workspace', { oldName, newName })
    const ws = workspaces.value.find(w => w.name === oldName)
    if (ws) {
      ws.name = newName
      const pathParts = ws.path.replace(/\\/g, '/').split('/')
      if (pathParts[pathParts.length - 1] === oldName) {
        pathParts[pathParts.length - 1] = newName
        ws.path = pathParts.join(ws.path.includes('\\') ? '\\' : '/')
      }
    }
    if (currentWorkspace.value?.name === oldName) {
      currentWorkspace.value.name = newName
    }
    if (lastWorkspaceName.value === oldName) {
      localStorage.setItem('pingport-last-workspace', newName)
      lastWorkspaceName.value = newName
    }
    // Refresh the list
    const result = await invoke<ScanResult>('scan_workspaces')
    workspaces.value = result.workspaces.map(transformWorkspace)
  }

  return {
    workspaces,
    currentWorkspace,
    isLoading,
    error,
    lastWorkspaceName,
    scanWorkspaces,
    switchWorkspace,
    renameWorkspace,
    createTemporaryWorkspace,
    refreshStoresForWorkspace,
  }
})
