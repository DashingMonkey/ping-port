<script setup lang="ts">
import { ref, computed, watch, onMounted, toRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCollectionsStore } from '../../stores/collections'
import { useRequestsStore } from '../../stores/requests'
import { useTabsStore } from '../../stores/tabs'
import TreeItem from './TreeItem.vue'
import EmptyState from '../common/EmptyState.vue'
import { confirmDelete } from '../../composables/useConfirm'
import { prompt } from '../../composables/usePrompt'
import type { Collection } from '../../stores/collections'
import type { Request } from '../../stores/requests'
import { DEFAULT_REQUEST_STATE, type HttpMethod } from '../../stores/types'

const collectionsStore = useCollectionsStore()
const requestsStore = useRequestsStore()
const tabsStore = useTabsStore()
const { t } = useI18n()

const isLoading = ref(false)

// Reactive reference to activeTabId using toRef
const activeTabIdRef = toRef(tabsStore, 'activeTabId')

// Sync selection with active tab - utility method
const syncSelectionWithActiveTab = () => {
  const currentTabId = tabsStore.activeTabId
  if (!currentTabId) {
    collectionsStore.selectedRequests.clear()
    collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
    return
  }

  const activeTab = tabsStore.tabs.find(t => t.id === currentTabId)
  if (!activeTab) {
    collectionsStore.selectedRequests.clear()
    collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
    return
  }

  // Check if the active tab's request has unsaved changes
  const draft = tabsStore.drafts.find(d => d.tabId === currentTabId)
  if (draft && draft.changesCount > 0) {
    // Has unsaved changes - don't select
    collectionsStore.selectedRequests.clear()
    collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
    return
  }

  // Sync selection to the active tab's request
  collectionsStore.selectedRequests.clear()
  if (activeTab.requestId) {
    collectionsStore.selectedRequests.add(activeTab.requestId)

    // Auto-expand parent collections to show the selected request
    const request = requestsStore.requests.find(r => r.id === activeTab.requestId)
    if (request) {
      collectionsStore.expandToCollection(request.collectionId)
    }
  }
  collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
}

const rootCollections = computed(() => {
  return collectionsStore.collections
    .filter(c => c.parentId === null)
    .sort((a, b) => a.name.localeCompare(b.name))
})

const getChildCollections = (parentId: string): Collection[] => {
  return collectionsStore.collections
    .filter(c => c.parentId === parentId)
    .sort((a, b) => a.name.localeCompare(b.name))
}

const getRequestsByCollection = (collectionId: string): Request[] => {
  return requestsStore.requests
    .filter(r => r.collectionId === collectionId)
    .sort((a, b) => a.name.localeCompare(b.name))
}

const toggleNode = (id: string) => {
  if (collectionsStore.expandedNodes.has(id)) {
    collectionsStore.expandedNodes.delete(id)
  } else {
    collectionsStore.expandedNodes.add(id)
  }
  collectionsStore.expandedNodes = new Set(collectionsStore.expandedNodes)
}

const handleSelectRequest = (request: Request) => {
  // Check if this request is already open in a tab
  const existingTab = tabsStore.tabs.find(t => t.requestId === request.id)
  if (existingTab) {
    tabsStore.setActiveTab(existingTab.id)
    // Clear multi-select and select only this request
    collectionsStore.selectedRequests.clear()
    collectionsStore.selectedRequests.add(request.id)
    collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
    return
  }
  const newTab = tabsStore.createTab(request.id, request.method, `${request.method} ${request.name}`)
  // Create draft for tracking unsaved changes
  tabsStore.createDraft(newTab.id, {
    method: request.method as HttpMethod,
    url: request.url,
    params: request.params ? JSON.parse(request.params) : [],
    headers: request.headers ? JSON.parse(request.headers) : [],
    body: request.body ? JSON.parse(request.body) : { type: 'none', content: '' },
    auth: request.auth ? JSON.parse(request.auth) : { type: 'none' },
    preRequestScript: request.preRequestScript ?? '',
    testScript: request.testScript ?? '',
  })
  // Clear multi-select and select only this request
  collectionsStore.selectedRequests.clear()
  collectionsStore.selectedRequests.add(request.id)
  collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
}

const handleDeleteCollection = async (id: string) => {
  if (await confirmDelete(t('collection.deleteConfirm'))) {
    await collectionsStore.deleteCollection(id)
  }
}

const handleRenameCollection = async (id: string) => {
  const collection = collectionsStore.collections.find(c => c.id === id)
  if (!collection) return
  const newName = await prompt(t('collection.enterNewName'), collection.name)
  if (newName && newName !== collection.name) {
    await collectionsStore.updateCollection({ id, name: newName })
  }
}

const handleDeleteRequest = async (id: string) => {
  if (await confirmDelete(t('collection.deleteRequestConfirm'))) {
    // Close tabs referencing this request
    const tabsToClose = tabsStore.tabs.filter(t => t.requestId === id)
    for (const tab of tabsToClose) {
      tabsStore.closeTab(tab.id)
    }
    await requestsStore.deleteRequest(id)
  }
}

const handleDeleteSelectedRequests = async () => {
  if (collectionsStore.selectedRequests.size === 0) return
  const count = collectionsStore.selectedRequests.size
  const message = count === 1
    ? t('collection.deleteRequestConfirm')
    : t('collection.deleteRequestsConfirm', { count })
  if (await confirmDelete(message)) {
    // Close tabs referencing deleted requests
    for (const id of collectionsStore.selectedRequests) {
      const tabsToClose = tabsStore.tabs.filter(t => t.requestId === id)
      for (const tab of tabsToClose) {
        tabsStore.closeTab(tab.id)
      }
    }
    for (const id of collectionsStore.selectedRequests) {
      await requestsStore.deleteRequest(id)
    }
    collectionsStore.selectedRequests.clear()
    collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
  }
}

const handleRequestSelect = (request: Request, event: MouseEvent) => {
  if (event.ctrlKey || event.metaKey) {
    // Toggle selection with Ctrl/Cmd+click
    if (collectionsStore.selectedRequests.has(request.id)) {
      collectionsStore.selectedRequests.delete(request.id)
    } else {
      collectionsStore.selectedRequests.add(request.id)
    }
    collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
  } else {
    // Normal click - select only this one and open it
    collectionsStore.selectedRequests.clear()
    collectionsStore.selectedRequests.add(request.id)
    collectionsStore.selectedRequests = new Set(collectionsStore.selectedRequests)
    // Also open the request
    handleSelectRequest(request)
  }
}

// Sync selection with active tab
watch(activeTabIdRef, () => {
  syncSelectionWithActiveTab()
}, { immediate: false })

const handleNewRequest = async (collectionId: string) => {
  // Create a new request under the specified collection
  const newRequest = await requestsStore.createRequest({
    name: t('collection.newRequest'),
    collectionId,
    method: DEFAULT_REQUEST_STATE.method,
    url: DEFAULT_REQUEST_STATE.url,
    params: JSON.stringify(DEFAULT_REQUEST_STATE.params),
    headers: JSON.stringify(DEFAULT_REQUEST_STATE.headers),
    body: JSON.stringify(DEFAULT_REQUEST_STATE.body),
    auth: JSON.stringify(DEFAULT_REQUEST_STATE.auth),
    preRequestScript: DEFAULT_REQUEST_STATE.preRequestScript,
    testScript: DEFAULT_REQUEST_STATE.testScript,
  })
  // Open the new request in a tab
  tabsStore.createTab(newRequest.id, newRequest.method, `${newRequest.method} ${newRequest.name}`)
  // Make sure the collection is expanded so the user can see the new request
  collectionsStore.expandedNodes.add(collectionId)
  collectionsStore.expandedNodes = new Set(collectionsStore.expandedNodes)
}

onMounted(async () => {
  isLoading.value = true
  try {
    await Promise.all([
      collectionsStore.fetchCollections(),
      requestsStore.fetchRequests(),
    ])
  } finally {
    isLoading.value = false
  }
  // Sync selection after data loads
  syncSelectionWithActiveTab()
})
</script>

<template>
  <div class="flex flex-col h-full">
    <div v-if="isLoading" class="flex items-center justify-center p-4">
      <div class="text-sm text-gray-500 dark:text-gray-400">{{ t('common.loading') }}</div>
    </div>

    <div v-else-if="rootCollections.length === 0" class="flex-1">
      <EmptyState
        :title="t('collection.noCollections')"
        :message="t('collection.noCollectionsMessage')"
      />
    </div>

    <div v-else class="flex-1 overflow-auto">
      <TreeItem
        v-for="collection in rootCollections"
        :key="collection.id"
        :collection="collection"
        :requests="getRequestsByCollection(collection.id)"
        :child-collections="getChildCollections(collection.id)"
        :all-collections="collectionsStore.collections"
        :all-requests="requestsStore.requests"
        :expanded-nodes="collectionsStore.expandedNodes"
        :level="0"
        :selected-requests="collectionsStore.selectedRequests"
        @toggle="toggleNode"
        @select-request="handleSelectRequest"
        @request-select="handleRequestSelect"
        @delete-collection="handleDeleteCollection"
        @rename-collection="handleRenameCollection"
        @delete-request="handleDeleteRequest"
        @delete-selected="handleDeleteSelectedRequests"
        @new-request="handleNewRequest"
      />
    </div>
  </div>
</template>
