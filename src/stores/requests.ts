import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Request {
  id: string
  collectionId: string
  name: string
  method: string
  url: string
  params: string | null
  headers: string | null
  body: string | null
  auth: string | null
  preRequestScript: string | null
  testScript: string | null
  position: number
  createdAt: string
  updatedAt: string
}

export interface CreateRequestInput {
  collectionId: string
  name: string
  method: string
  url: string
  params?: string
  headers?: string
  body?: string
  auth?: string
  preRequestScript?: string
  testScript?: string
}

export interface UpdateRequestInput {
  id: string
  collectionId?: string
  name?: string
  method?: string
  url?: string
  params?: string
  headers?: string
  body?: string
  auth?: string
  preRequestScript?: string
  testScript?: string
}

interface RustRequest {
  id: string
  collection_id: string
  name: string
  method: string
  url: string
  params: string | null
  headers: string | null
  body: string | null
  auth: string | null
  pre_request_script: string | null
  test_script: string | null
  position: number
  created_at: string
  updated_at: string
}

function transformFromRust(req: RustRequest): Request {
  return {
    id: req.id,
    collectionId: req.collection_id,
    name: req.name,
    method: req.method,
    url: req.url,
    params: req.params,
    headers: req.headers,
    body: req.body,
    auth: req.auth,
    preRequestScript: req.pre_request_script,
    testScript: req.test_script,
    position: req.position,
    createdAt: req.created_at,
    updatedAt: req.updated_at,
  }
}

export const useRequestsStore = defineStore('requests', () => {
  const requests = ref<Request[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const fetchRequests = async () => {
    isLoading.value = true
    error.value = null
    try {
      const result = await invoke<RustRequest[]>('get_requests')
      requests.value = result.map(transformFromRust)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  const createRequest = async (input: CreateRequestInput): Promise<Request> => {
    try {
      const rustInput = {
        collection_id: input.collectionId,
        name: input.name,
        method: input.method,
        url: input.url,
        params: input.params || null,
        headers: input.headers || null,
        body: input.body || null,
        auth: input.auth || null,
        pre_request_script: input.preRequestScript || null,
        test_script: input.testScript || null,
      }
      const result = await invoke<RustRequest>('create_request', { input: rustInput })
      const request = transformFromRust(result)
      requests.value.push(request)
      return request
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  const updateRequest = async (input: UpdateRequestInput): Promise<void> => {
    try {
      const rustInput = {
        id: input.id,
        collection_id: input.collectionId !== undefined ? input.collectionId : null,
        name: input.name !== undefined ? input.name : null,
        method: input.method !== undefined ? input.method : null,
        url: input.url !== undefined ? input.url : null,
        params: input.params !== undefined ? input.params : null,
        headers: input.headers !== undefined ? input.headers : null,
        body: input.body !== undefined ? input.body : null,
        auth: input.auth !== undefined ? input.auth : null,
        pre_request_script: input.preRequestScript !== undefined ? input.preRequestScript : null,
        test_script: input.testScript !== undefined ? input.testScript : null,
      }
      await invoke('update_request', { input: rustInput })
      const index = requests.value.findIndex(r => r.id === input.id)
      if (index !== -1) {
        const existing = requests.value[index]
        requests.value[index] = {
          ...existing,
          collectionId: input.collectionId ?? existing.collectionId,
          name: input.name ?? existing.name,
          method: input.method ?? existing.method,
          url: input.url ?? existing.url,
          params: input.params !== undefined ? input.params : existing.params,
          headers: input.headers !== undefined ? input.headers : existing.headers,
          body: input.body !== undefined ? input.body : existing.body,
          auth: input.auth !== undefined ? input.auth : existing.auth,
          preRequestScript: input.preRequestScript !== undefined ? input.preRequestScript : existing.preRequestScript,
          testScript: input.testScript !== undefined ? input.testScript : existing.testScript,
          updatedAt: new Date().toISOString(),
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  const deleteRequest = async (id: string): Promise<void> => {
    try {
      await invoke('delete_request', { id })
      requests.value = requests.value.filter(r => r.id !== id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  const getByCollection = (collectionId: string): Request[] => {
    return requests.value.filter(r => r.collectionId === collectionId)
  }

  const reorderRequests = async (sourceId: string, targetId: string): Promise<void> => {
    // Optimistically update local state
    const sourceRequest = requests.value.find(r => r.id === sourceId)
    const targetRequest = requests.value.find(r => r.id === targetId)
    if (!sourceRequest || !targetRequest) return

    const collectionRequests = requests.value.filter(r => r.collectionId === sourceRequest.collectionId)
    collectionRequests.sort((a, b) => a.position - b.position)

    const sourceIndex = collectionRequests.findIndex(r => r.id === sourceId)
    const targetIndex = collectionRequests.findIndex(r => r.id === targetId)

    if (sourceIndex === -1 || targetIndex === -1) return

    const [removed] = collectionRequests.splice(sourceIndex, 1)
    collectionRequests.splice(targetIndex, 0, removed)

    collectionRequests.forEach((req, index) => {
      req.position = index
    })

    // Persist to backend
    try {
      await invoke('reorder_requests', { input: { source_id: sourceId, target_id: targetId } })
    } catch (e) {
      error.value = `Failed to reorder requests: ${e}`
      await fetchRequests()
    }
  }

  return {
    requests,
    isLoading,
    error,
    fetchRequests,
    createRequest,
    updateRequest,
    deleteRequest,
    getByCollection,
    reorderRequests,
  }
})
