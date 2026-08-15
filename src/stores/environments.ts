import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Environment {
  id: string
  name: string
  variables: Record<string, string>
}

export interface CreateEnvironmentInput {
  name: string
  variables?: Record<string, string>
}

export interface UpdateEnvironmentInput {
  id: string
  name?: string
  variables?: Record<string, string>
}

interface RustEnvironment {
  id: string
  name: string
  variables: Record<string, string>
}

function transformFromRust(env: RustEnvironment): Environment {
  return {
    id: env.id,
    name: env.name,
    variables: env.variables,
  }
}

const SETTINGS_KEY = 'pingport-env-settings'

interface PingPortSettings {
  activeEnvId?: string | null
}

function loadSettings(): PingPortSettings {
  const stored = localStorage.getItem(SETTINGS_KEY)
  return stored ? JSON.parse(stored) : {}
}

function saveSettings(settings: PingPortSettings) {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
}

export const useEnvironmentsStore = defineStore('environments', () => {
  const environments = ref<Environment[]>([])
  const globalVariables = ref<Record<string, string>>({})
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Active environment ID stored in localStorage (global, not per-workspace)
  const activeEnvId = ref<string | null>(loadSettings().activeEnvId || null)

  // Collection preferences (collectionId -> envId)
  const collectionPreferences = ref<Record<string, string>>({})
  // Tab overrides (tabId -> envId)
  const tabOverrides = ref<Record<string, string>>({})

  // Active environment object
  const activeEnvironment = computed(() => {
    if (!activeEnvId.value) return null
    return environments.value.find(env => env.id === activeEnvId.value) || null
  })

  // Get environment by ID
  const getEnvironmentById = (envId: string): Environment | undefined => {
    return environments.value.find(env => env.id === envId)
  }

  // Resolve active environment based on priority: tab > collection > global
  const resolveActiveEnvId = (tabId: string | null, collectionId: string | null): string => {
    // 1. Check tab override (highest priority)
    if (tabId && tabOverrides.value[tabId]) {
      return tabOverrides.value[tabId]
    }

    // 2. Check collection preference (medium priority)
    if (collectionId && collectionPreferences.value[collectionId]) {
      return collectionPreferences.value[collectionId]
    }

    // 3. Fall back to user-selected environment
    return activeEnvId.value || 'default'
  }

  // Get the resolved active environment object
  const resolvedActiveEnvironment = computed(() => {
    // This is a placeholder - actual resolution happens in RequestPanel with tab/collection context
    return activeEnvironment.value
  })

  const getGlobalVariables = (): Record<string, string> => {
    const globalEnv = environments.value.find(e => e.id === '__global__')
    return globalEnv?.variables ?? {}
  }

  // Get effective variables for a specific tab and collection
  const getEffectiveVariables = (tabId: string | null, collectionId: string | null): Record<string, string> => {
    const envId = resolveActiveEnvId(tabId, collectionId)
    const env = getEnvironmentById(envId)

    const merged: Record<string, string> = { ...getGlobalVariables() }
    if (env && env.id !== '__global__') {
      Object.assign(merged, env.variables)
    }
    return merged
  }

  // Legacy computed for backward compatibility
  const getVariables = computed(() => {
    const merged: Record<string, string> = { ...getGlobalVariables() }
    if (activeEnvironment.value && activeEnvironment.value.id !== '__global__') {
      Object.assign(merged, activeEnvironment.value.variables)
    }
    return merged
  })

  const fetchEnvironments = async () => {
    isLoading.value = true
    error.value = null
    try {
      const result = await invoke<RustEnvironment[]>('get_environments')
      let envs = result.map(transformFromRust)

      // Ensure __global__ exists
      const hasGlobal = envs.find(e => e.id === '__global__')
      if (!hasGlobal) {
        await invoke('create_environment', {
          input: {
            id: '__global__',
            name: 'Global Variables',
            variables: {},
          }
        })
        envs.push({ id: '__global__', name: 'Global Variables', variables: {} })
      }

      environments.value = envs.sort((a, b) => {
        // __global__ always last
        if (a.id === '__global__') return 1
        if (b.id === '__global__') return -1
        return a.name.localeCompare(b.name)
      })
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  const createEnvironment = async (input: CreateEnvironmentInput): Promise<Environment> => {
    try {
      const rustInput = {
        name: input.name,
        variables: input.variables || {},
      }
      const result = await invoke<RustEnvironment>('create_environment', { input: rustInput })
      const environment = transformFromRust(result)
      environments.value.push(environment)
      return environment
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  const updateEnvironment = async (input: UpdateEnvironmentInput): Promise<void> => {
    try {
      const rustInput = {
        id: input.id,
        name: input.name !== undefined ? input.name : null,
        variables: input.variables !== undefined ? input.variables : null,
      }
      await invoke('update_environment', { input: rustInput })
      const index = environments.value.findIndex(env => env.id === input.id)
      if (index !== -1) {
        const existing = environments.value[index]
        environments.value[index] = {
          ...existing,
          name: input.name ?? existing.name,
          variables: input.variables !== undefined ? input.variables : existing.variables,
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  const deleteEnvironment = async (id: string): Promise<void> => {
    try {
      await invoke('delete_environment', { id })
      environments.value = environments.value.filter(env => env.id !== id)

      // If deleted env was active, clear active
      if (activeEnvId.value === id) {
        setActiveEnvironment(null)
      }

      // Clean up collection preferences
      Object.keys(collectionPreferences.value).forEach(colId => {
        if (collectionPreferences.value[colId] === id) {
          delete collectionPreferences.value[colId]
        }
      })

      // Clean up tab overrides
      Object.keys(tabOverrides.value).forEach(tabId => {
        if (tabOverrides.value[tabId] === id) {
          delete tabOverrides.value[tabId]
        }
      })
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  const deleteEnvironmentVariable = async (envId: string, key: string): Promise<void> => {
    try {
      await invoke('delete_environment_variable', { envId, key })
      const env = environments.value.find(e => e.id === envId)
      if (env) {
        delete env.variables[key]
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  const setActiveEnvironment = (id: string | null): void => {
    activeEnvId.value = id
    const settings = loadSettings()
    settings.activeEnvId = id
    saveSettings(settings)
  }

  // ============= Three-tier Environment Binding (local only) =============

  // Set collection preference (local only, no backend persistence)
  const setCollectionPreference = (collectionId: string, envId: string | null) => {
    if (envId) {
      collectionPreferences.value[collectionId] = envId
    } else {
      delete collectionPreferences.value[collectionId]
    }
  }

  // Set tab override (local only, no backend persistence)
  const setTabOverride = (tabId: string, envId: string | null) => {
    if (envId) {
      tabOverrides.value[tabId] = envId
    } else {
      delete tabOverrides.value[tabId]
    }
  }

  // Resolve environment locally (tab_id, collection_id) -> env_id
  const resolveEnvironment = (tabId: string | null, collectionId: string | null): string => {
    return resolveActiveEnvId(tabId, collectionId)
  }

  return {
    environments,
    globalVariables,
    activeEnvironment,
    resolvedActiveEnvironment,
    getVariables,
    collectionPreferences,
    tabOverrides,
    isLoading,
    error,
    fetchEnvironments,
    createEnvironment,
    updateEnvironment,
    deleteEnvironment,
    deleteEnvironmentVariable,
    setActiveEnvironment,
    setCollectionPreference,
    setTabOverride,
    resolveEnvironment,
    getEnvironmentById,
    resolveActiveEnvId,
    getEffectiveVariables,
  }
})
