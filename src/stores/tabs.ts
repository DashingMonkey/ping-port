import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { Tab, Draft, RequestState, RequestSubTab } from './types'

const getTabsKey = () => {
  const wsName = localStorage.getItem('pingport-last-workspace') || 'default'
  return `pingport-${wsName}-tabs`
}

const getDraftsKey = () => {
  const wsName = localStorage.getItem('pingport-last-workspace') || 'default'
  return `pingport-${wsName}-drafts`
}

export const useTabsStore = defineStore('tabs', () => {
  const tabs = ref<Tab[]>([])
  const drafts = ref<Draft[]>([])
  const activeTabId = ref<string | null>(null)
  // Session-only memory map: remembers the selected sub-tab per top-level tab.
  // Intentionally NOT persisted — resets to default on app restart.
  const subTabs = ref<Record<string, RequestSubTab>>({})

  const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value) || null)
  const activeDraft = computed(() => {
    if (!activeTabId.value) return null
    return drafts.value.find(d => d.tabId === activeTabId.value) || null
  })

  const loadTabs = () => {
    const key = getTabsKey()
    const stored = localStorage.getItem(key)
    if (stored) {
      try {
        const parsed = JSON.parse(stored)
        tabs.value = parsed.tabs || []
        activeTabId.value = parsed.activeTabId || null
      } catch (e) {
        console.warn('Failed to parse tabs:', e)
      }
    }
    const draftsKey = getDraftsKey()
    const draftsStored = localStorage.getItem(draftsKey)
    if (draftsStored) {
      try {
        drafts.value = JSON.parse(draftsStored)
      } catch (e) {
        console.warn('Failed to parse drafts:', e)
      }
    }
  }

  const createTab = (requestId: string, method: string, title: string) => {
    const maxPosition = tabs.value.reduce((max, t) => Math.max(max, t.position), 0)
    const tab: Tab = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      requestId,
      method,
      title,
      position: maxPosition + 1,
    }

    tabs.value.push(tab)
    activeTabId.value = tab.id

    return tab
  }

  const closeTab = (tabId: string) => {
    const index = tabs.value.findIndex(t => t.id === tabId)
    if (index === -1) return

    tabs.value.splice(index, 1)

    // Remove associated draft
    const draftIndex = drafts.value.findIndex(d => d.tabId === tabId)
    if (draftIndex !== -1) {
      drafts.value.splice(draftIndex, 1)
    }

    // Clean up remembered sub-tab
    delete subTabs.value[tabId]

    // Update activeTabId if closing the active tab
    if (activeTabId.value === tabId) {
      if (tabs.value[index - 1]) {
        activeTabId.value = tabs.value[index - 1].id
      } else if (tabs.value[0]) {
        activeTabId.value = tabs.value[0].id
      } else {
        activeTabId.value = null
      }
    }
  }

  const setActiveTab = (tabId: string) => {
    activeTabId.value = tabId
  }

  const getSubTab = (tabId: string): RequestSubTab => subTabs.value[tabId] ?? 'params'

  const setSubTab = (tabId: string, subTab: RequestSubTab) => {
    subTabs.value[tabId] = subTab
  }

  const updateTabTitle = (tabId: string, title: string) => {
    const tab = tabs.value.find(t => t.id === tabId)
    if (tab) {
      tab.title = title
    }
  }

  const bindTabToRequest = (tabId: string, requestId: string, method: string) => {
    const tab = tabs.value.find(t => t.id === tabId)
    if (tab) {
      tab.requestId = requestId
      tab.method = method
    }
  }

  const createDraft = (tabId: string, initialState: RequestState): Draft => {
    const draft: Draft = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      tabId,
      sourceType: 'new',
      state: initialState,
      changesCount: 0,
    }
    drafts.value.push(draft)
    return draft
  }

  const markDirty = (draftId: string) => {
    const draft = drafts.value.find(d => d.id === draftId)
    if (draft) {
      draft.changesCount++
    }
  }

  const updateDraftState = (draftId: string, state: RequestState) => {
    const draft = drafts.value.find(d => d.id === draftId)
    if (draft) {
      draft.state = state
    }
  }

  const markClean = (draftId: string) => {
    const draft = drafts.value.find(d => d.id === draftId)
    if (draft) {
      draft.changesCount = 0
      draft.originalState = JSON.parse(JSON.stringify(draft.state))
    }
  }

  const removeDraft = (draftId: string) => {
    const index = drafts.value.findIndex(d => d.id === draftId)
    if (index !== -1) {
      drafts.value.splice(index, 1)
    }
  }

  const hasUnsavedChanges = (draftId: string): boolean => {
    const draft = drafts.value.find(d => d.id === draftId)
    return draft ? draft.changesCount > 0 : false
  }

  // Auto-save tabs and drafts to localStorage (debounced)
  let saveTimer: ReturnType<typeof setTimeout> | null = null
  const persist = () => {
    if (saveTimer) {
      clearTimeout(saveTimer)
      saveTimer = null
    }
    localStorage.setItem(getTabsKey(), JSON.stringify({ tabs: tabs.value, activeTabId: activeTabId.value }))
    localStorage.setItem(getDraftsKey(), JSON.stringify(drafts.value))
  }
  watch([tabs, drafts, activeTabId], () => {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(persist, 500)
  }, { flush: 'post', deep: true })

  // Flush pending save when app is closing (debounce window would otherwise lose data)
  window.addEventListener('beforeunload', persist)
  window.addEventListener('pagehide', persist)

  return {
    tabs,
    drafts,
    activeTabId,
    activeTab,
    activeDraft,
    loadTabs,
    createTab,
    closeTab,
    setActiveTab,
    getSubTab,
    setSubTab,
    updateTabTitle,
    bindTabToRequest,
    createDraft,
    markDirty,
    updateDraftState,
    markClean,
    removeDraft,
    hasUnsavedChanges,
  }
})
