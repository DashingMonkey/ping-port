<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTabsStore } from '../../stores/tabs'
import { DEFAULT_REQUEST_STATE, type HttpMethod } from '../../stores/types'
import { useRequestsStore } from '../../stores/requests'

const { t } = useI18n()
const tabsStore = useTabsStore()
const requestsStore = useRequestsStore()

const contextMenu = ref<{ show: boolean; x: number; y: number; tabId: string | null }>({
  show: false,
  x: 0,
  y: 0,
  tabId: null,
})

const methodColors: Record<string, string> = {
  GET: 'bg-success/15 text-success',
  POST: 'bg-info/15 text-info',
  PUT: 'bg-warning/15 text-warning',
  DELETE: 'bg-error/15 text-error',
  PATCH: 'bg-purple-400/15 text-purple-400',
  OPTIONS: 'bg-surface-elevated text-text-muted',
}

const methodAbbr: Record<string, string> = {
  GET: 'get',
  POST: 'post',
  PUT: 'put',
  DELETE: 'del',
  PATCH: 'patch',
  OPTIONS: 'opt',
}

const getMethodColor = (method: string): string => {
  return methodColors[method] || methodColors.GET
}

const getMethodAbbr = (method: string): string => {
  return methodAbbr[method] || method.toLowerCase()
}

const getTabRequest = (tab: typeof tabsStore.tabs[0]) => {
  if (tab.requestId) {
    return requestsStore.requests.find(r => r.id === tab.requestId)
  }
  return null
}

const handleNewTab = () => {
  const tab = tabsStore.createTab('', 'GET', t('tabs.newRequest'))
  tabsStore.createDraft(tab.id, { ...DEFAULT_REQUEST_STATE })
}

const handleCloseTab = (tabId: string, event: Event) => {
  event.stopPropagation()
  tabsStore.closeTab(tabId)
}

const handleTabClick = (tabId: string) => {
  tabsStore.setActiveTab(tabId)
}

const handleContextMenu = (event: MouseEvent, tabId: string) => {
  event.preventDefault()
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    tabId,
  }
}

const closeContextMenu = () => {
  contextMenu.value.show = false
}

function handleContextmenuKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    closeContextMenu()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleContextmenuKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleContextmenuKeydown)
})

const handleCloseThis = () => {
  if (contextMenu.value.tabId) {
    tabsStore.closeTab(contextMenu.value.tabId)
  }
  closeContextMenu()
}

const handleCloseOthers = () => {
  if (contextMenu.value.tabId) {
    const tabsToClose = tabsStore.tabs.filter(t => t.id !== contextMenu.value.tabId)
    tabsToClose.forEach(t => tabsStore.closeTab(t.id))
  }
  closeContextMenu()
}

const handleCloseAll = () => {
  ;[...tabsStore.tabs].reverse().forEach(t => tabsStore.closeTab(t.id))
  closeContextMenu()
}

const handleCloseLeft = () => {
  if (contextMenu.value.tabId) {
    const currentIndex = tabsStore.tabs.findIndex(t => t.id === contextMenu.value.tabId)
    const leftTabs = tabsStore.tabs.slice(0, currentIndex)
    ;[...leftTabs].reverse().forEach(t => tabsStore.closeTab(t.id))
  }
  closeContextMenu()
}

const handleCloseRight = () => {
  if (contextMenu.value.tabId) {
    const currentIndex = tabsStore.tabs.findIndex(t => t.id === contextMenu.value.tabId)
    const rightTabs = tabsStore.tabs.slice(currentIndex + 1)
    ;[...rightTabs].reverse().forEach(t => tabsStore.closeTab(t.id))
  }
  closeContextMenu()
}

const handleDuplicate = () => {
  if (!contextMenu.value.tabId) return
  const tab = tabsStore.tabs.find(t => t.id === contextMenu.value.tabId)
  if (!tab || !tab.requestId) return

  const originalRequest = requestsStore.requests.find(r => r.id === tab.requestId)
  if (!originalRequest) return

  const newTab = tabsStore.createTab('', originalRequest.method, t('tabs.copyOf', { name: originalRequest.name }))
  tabsStore.createDraft(newTab.id, {
    method: originalRequest.method as HttpMethod,
    url: originalRequest.url,
    params: originalRequest.params ? JSON.parse(originalRequest.params) : [],
    headers: originalRequest.headers ? JSON.parse(originalRequest.headers) : [],
    body: originalRequest.body ? JSON.parse(originalRequest.body) : { type: 'none', content: '' },
    auth: originalRequest.auth ? JSON.parse(originalRequest.auth) : { type: 'none' },
    preRequestScript: originalRequest.preRequestScript ?? '',
    testScript: originalRequest.testScript ?? '',
  })
  closeContextMenu()
}

const isRequestTab = (tabId: string | null) => {
  if (!tabId) return false
  const tab = tabsStore.tabs.find(t => t.id === tabId)
  return !!tab?.requestId
}

const isTabDirty = (tabId: string | null) => {
  if (!tabId) return false
  const draft = tabsStore.drafts.find(d => d.tabId === tabId)
  return draft ? draft.changesCount > 0 : false
}
</script>

<template>
  <div class="flex bg-surface-deep h-8 px-0 tab-bar border-b border-border-default">
    <!-- Tabs -->
    <div
      v-for="tab in tabsStore.tabs"
      :key="tab.id"
      @click="handleTabClick(tab.id)"
      @contextmenu="handleContextMenu($event, tab.id)" data-contextmenu
      @mouseup="(e) => e.button === 1 && handleCloseTab(tab.id, e)"
      class="group flex items-center gap-1 px-2 text-xs cursor-pointer border-r border-border-default min-w-0 max-w-[200px] transition-colors duration-150"
      :class="tab.id === tabsStore.activeTabId
        ? 'bg-surface-base text-text-primary border-t-2 border-t-accent h-[calc(100%+1px)]'
        : 'h-full text-text-muted hover:text-text-secondary hover:bg-surface-elevated'"
    >
      <span v-if="getTabRequest(tab)" class="truncate flex items-center gap-1">
        <span
          class="inline-block px-1 py-0 text-[10px] font-semibold rounded uppercase font-['JetBrains_Mono']"
          :class="getMethodColor(getTabRequest(tab)!.method)"
        >
          {{ getMethodAbbr(getTabRequest(tab)!.method) }}
        </span>
        <span class="text-[11px]">{{ getTabRequest(tab)!.name }}</span>
      </span>
      <span v-else class="truncate flex items-center gap-1 font-['JetBrains_Mono'] text-[11px]">
        {{ tab.title }}
      </span>
      <!-- Dirty indicator and close button occupy the same space -->
      <div class="relative w-5 h-5 flex-shrink-0 flex items-center justify-center">
        <!-- Dirty dot (shown when dirty, hidden on hover) -->
        <span
          v-if="isTabDirty(tab.id)"
          class="absolute w-1.5 h-1.5 rounded-full bg-accent group-hover:hidden"
        ></span>
        <!-- Close button (shown on hover, always present in DOM) -->
        <button
          @click="handleCloseTab(tab.id, $event)"
          class="absolute p-0.5 rounded opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:bg-surface-elevated transition-opacity duration-150"
          :title="t('tabs.closeTab')"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>
    </div>

    <!-- New Tab Button -->
    <button
      @click="handleNewTab"
      class="flex items-center justify-center w-8 h-8 ml-1 rounded hover:bg-surface-elevated text-text-muted hover:text-accent transition-colors duration-150"
      :title="t('tabs.newRequestTooltip')"
    >
      <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
      </svg>
    </button>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenu.show"
        class="fixed z-50 bg-surface-base border border-border-default rounded-lg shadow-lg py-1 min-w-[160px]"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
        @click.stop
        role="menu"
      >
        <button
          v-if="isRequestTab(contextMenu.tabId)"
          class="w-full px-4 py-2 text-left text-xs text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="handleDuplicate"
        >
          {{ t('tabs.duplicate') }}
        </button>
        <div v-if="isRequestTab(contextMenu.tabId)" class="h-px bg-border-default my-1"></div>
        <button
          class="w-full px-4 py-2 text-left text-xs text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="handleCloseThis"
        >
          {{ t('tabs.close') }}
        </button>
        <button
          class="w-full px-4 py-2 text-left text-xs text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="handleCloseOthers"
        >
          {{ t('tabs.closeOtherTabs') }}
        </button>
        <button
          class="w-full px-4 py-2 text-left text-xs text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="handleCloseAll"
          role="menuitem"
        >
          {{ t('tabs.closeAllTabs') }}
        </button>
        <div class="h-px bg-border-default my-1"></div>
        <button
          class="w-full px-4 py-2 text-left text-xs text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="handleCloseLeft"
        >
          {{ t('tabs.closeTabsLeft') }}
        </button>
        <button
          class="w-full px-4 py-2 text-left text-xs text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="handleCloseRight"
          role="menuitem"
        >
          {{ t('tabs.closeTabsRight') }}
        </button>
      </div>
      <div
        v-if="contextMenu.show"
        class="fixed inset-0 z-40"
        @click="closeContextMenu"
        @contextmenu.prevent="closeContextMenu"
      ></div>
    </Teleport>
  </div>
</template>
