import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Collection {
  id: string
  parentId: string | null
  name: string
  type: string
  createdAt: string
  updatedAt: string
  position: number
}

export interface CreateCollectionInput {
  name: string
  type?: string
  parentId?: string
}

export interface UpdateCollectionInput {
  id: string
  name?: string
  type?: string
  parentId?: string
}

interface RustCollection {
  id: string
  name: string
  kind: string | null
  parent_id: string | null
  position: number
  created_at: string
  updated_at: string
}

function transformFromRust(col: RustCollection): Collection {
  return {
    id: col.id,
    parentId: col.parent_id,
    name: col.name,
    type: col.kind || 'folder',
    position: col.position,
    createdAt: col.created_at,
    updatedAt: col.updated_at,
  }
}

export const useCollectionsStore = defineStore('collections', () => {
  const collections = ref<Collection[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const expandedNodes = ref<Set<string>>(new Set())
  const selectedRequests = ref<Set<string>>(new Set())

  // Expand all parent collections of a given collection
  const expandToCollection = (collectionId: string) => {
    const collection = collections.value.find(c => c.id === collectionId)
    if (!collection) return

    // Add this collection to expanded nodes
    expandedNodes.value.add(collectionId)
    expandedNodes.value = new Set(expandedNodes.value)

    // Recursively expand parents
    if (collection.parentId) {
      expandToCollection(collection.parentId)
    }
  }

  const fetchCollections = async () => {
    isLoading.value = true
    error.value = null
    try {
      const result = await invoke<RustCollection[]>('get_collections')
      collections.value = result.map(transformFromRust).sort((a, b) => a.name.localeCompare(b.name))
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  const createCollection = async (input: CreateCollectionInput): Promise<Collection> => {
    const rustInput = {
      name: input.name,
      kind: input.type || null,
      parent_id: input.parentId || null,
    }
    const result = await invoke<RustCollection>('create_collection', { input: rustInput })
    const collection = transformFromRust(result)
    collections.value.unshift(collection)
    return collection
  }

  const updateCollection = async (input: UpdateCollectionInput): Promise<void> => {
    const rustInput = {
      id: input.id,
      name: input.name || null,
      kind: input.type !== undefined ? input.type : null,
      parent_id: input.parentId !== undefined ? input.parentId : null,
    }
    await invoke('update_collection', { input: rustInput })
    const index = collections.value.findIndex(c => c.id === input.id)
    if (index !== -1) {
      const existing = collections.value[index]
      collections.value[index] = {
        ...existing,
        name: input.name ?? existing.name,
        parentId: input.parentId !== undefined ? input.parentId : existing.parentId,
        type: input.type !== undefined ? (input.type || 'folder') : existing.type,
        updatedAt: new Date().toISOString(),
      }
    }
  }

  const deleteCollection = async (id: string): Promise<void> => {
    await invoke('delete_collection', { id })
    collections.value = collections.value.filter(c => c.id !== id)
  }

  const getChildren = (parentId: string | null): Collection[] => {
    return collections.value.filter(c => c.parentId === parentId)
  }

  const reorderCollections = async (sourceId: string, targetId: string, parentId: string | null): Promise<void> => {
    // Optimistically update local state
    const siblings = collections.value.filter(c => c.parentId === parentId)
    siblings.sort((a, b) => a.position - b.position)

    const sourceIndex = siblings.findIndex(c => c.id === sourceId)
    const targetIndex = siblings.findIndex(c => c.id === targetId)

    if (sourceIndex === -1 || targetIndex === -1) return

    // Remove source from current position
    const [removed] = siblings.splice(sourceIndex, 1)
    // Insert source at target position
    siblings.splice(targetIndex, 0, removed)

    // Update positions
    siblings.forEach((col, index) => {
      const collection = collections.value.find(c => c.id === col.id)
      if (collection) {
        collection.position = index
      }
    })

    // Persist to backend
    try {
      await invoke('reorder_collections', { input: { source_id: sourceId, target_id: targetId } })
    } catch (e) {
      error.value = `Failed to reorder collections: ${e}`
      await fetchCollections()
    }
  }

  return {
    collections,
    isLoading,
    error,
    expandedNodes,
    selectedRequests,
    expandToCollection,
    fetchCollections,
    createCollection,
    updateCollection,
    deleteCollection,
    getChildren,
    reorderCollections,
  }
})
