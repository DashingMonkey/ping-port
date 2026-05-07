<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Collection } from '../../stores/collections'
import type { Request } from '../../stores/requests'
import ContextMenu from './ContextMenu.vue'
import type { MenuItem } from './ContextMenu.vue'

const { t } = useI18n()
const props = defineProps<{
  collection: Collection
  requests: Request[]
  childCollections: Collection[]
  allCollections: Collection[]
  allRequests: Request[]
  expandedNodes: Set<string>
  level: number
  selectedRequests?: Set<string>
}>()

const emit = defineEmits<{
  toggle: [id: string]
  selectRequest: [request: Request]
  'request-select': [request: Request, event: MouseEvent]
  'delete-collection': [id: string]
  'rename-collection': [id: string]
  'delete-request': [id: string]
  'delete-selected': []
  'new-request': [collectionId: string]
}>()

const contextMenu = ref<{ show: boolean; x: number; y: number }>({
  show: false,
  x: 0,
  y: 0,
})

const requestContextMenu = ref<{ show: boolean; x: number; y: number; request: Request | null }>({
  show: false,
  x: 0,
  y: 0,
  request: null,
})

const contextMenuItems = computed<MenuItem[]>(() => [
  {
    label: t('collection.newRequest'),
    action: () => emit('new-request', props.collection.id),
  },
  {
    label: t('collection.rename'),
    action: () => emit('rename-collection', props.collection.id),
  },
  {
    label: t('collection.delete'),
    action: () => emit('delete-collection', props.collection.id),
    danger: true,
  },
])

const handleContextMenu = (event: MouseEvent) => {
  event.preventDefault()
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
  }
}

const closeContextMenu = () => {
  contextMenu.value.show = false
}

const closeRequestContextMenu = () => {
  requestContextMenu.value.show = false
}

const handleRequestContextMenu = (event: MouseEvent, request: Request) => {
  event.preventDefault()
  requestContextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    request,
  }
}

const requestContextMenuItems = computed<MenuItem[]>(() => {
  const items: MenuItem[] = []

  if (props.selectedRequests && props.selectedRequests.size > 1) {
    items.push({
      label: t('collection.deleteSelected', { count: props.selectedRequests.size }),
      action: () => emit('delete-selected'),
      danger: true,
    })
  } else if (requestContextMenu.value.request) {
    items.push({
      label: t('collection.delete'),
      action: () => emit('delete-request', requestContextMenu.value.request!.id),
      danger: true,
    })
  }

  return items
})

const isExpanded = computed(() => props.expandedNodes.has(props.collection.id))

const hasChildren = computed(() => {
  return props.childCollections.length > 0 || props.requests.length > 0
})

const methodColors: Record<string, string> = {
  GET: 'bg-success/15 text-success',
  POST: 'bg-info/15 text-info',
  PUT: 'bg-warning/15 text-warning',
  DELETE: 'bg-error/15 text-error',
  PATCH: 'bg-purple-400/15 text-purple-400',
  OPTIONS: 'bg-surface-elevated text-text-muted',
  HEAD: 'bg-surface-elevated text-text-muted',
}

const methodAbbr: Record<string, string> = {
  GET: 'get',
  POST: 'post',
  PUT: 'put',
  DELETE: 'del',
  PATCH: 'patch',
  OPTIONS: 'opt',
  HEAD: 'head',
}

const getMethodColor = (method: string): string => {
  return methodColors[method] || methodColors.GET
}

const getMethodAbbr = (method: string): string => {
  return methodAbbr[method] || method.toLowerCase()
}

const handleToggle = () => {
  if (hasChildren.value) {
    emit('toggle', props.collection.id)
  }
}

const handleRequestClick = (request: Request, event: MouseEvent) => {
  if (event.ctrlKey || event.metaKey) {
    emit('request-select', request, event)
  } else {
    emit('selectRequest', request)
  }
}

const isRequestSelected = (requestId: string): boolean => {
  return props.selectedRequests?.has(requestId) ?? false
}

const getChildCollections = (parentId: string): Collection[] => {
  return props.allCollections.filter(c => c.parentId === parentId)
}

const getRequestsByCollection = (parentId: string): Request[] => {
  return props.allRequests.filter(r => r.collectionId === parentId).sort((a, b) => a.name.localeCompare(b.name))
}
</script>

<template>
  <div class="select-none">
    <!-- Collection Item -->
    <div
      class="flex items-center gap-1 py-1.5 cursor-pointer transition-colors duration-100 hover:bg-surface-elevated"
      :style="{ paddingLeft: `${level * 16 + 8}px`, paddingRight: '8px' }"
      @click.stop="handleToggle"
      @contextmenu="handleContextMenu" data-contextmenu
    >
      <!-- Expand/Collapse Chevron -->
      <span
        v-if="hasChildren"
        class="w-4 h-4 flex-shrink-0 text-text-muted transition-transform duration-100"
        :class="{ 'rotate-90': isExpanded }"
      >
        <svg viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
        </svg>
      </span>
      <span v-else class="w-4 h-4 flex-shrink-0"></span>

      <!-- Collection Icon -->
      <span class="w-4 h-4 flex-shrink-0 text-text-muted">
        <svg v-if="collection.type === 'collection'" viewBox="0 0 20 20" fill="currentColor">
          <path d="M7 3a1 1 0 000 2h6a1 1 0 100-2H7zM4 7a1 1 0 011-1h10a1 1 0 110 2H5a1 1 0 01-1-1zM2 11a2 2 0 012-2h12a2 2 0 012 2v4a2 2 0 01-2 2H4a2 2 0 01-2-2v-4z" />
        </svg>
        <svg v-else viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M2 6a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1H8a3 3 0 00-3 3v1.5a1.5 1.5 0 01-3 0V6z" clip-rule="evenodd" />
          <path d="M6 12a2 2 0 012-2h8a2 2 0 012 2v2a2 2 0 01-2 2H2h2a2 2 0 002-2v-2z" />
        </svg>
      </span>

      <!-- Collection Name -->
      <span class="text-xs leading-none text-text-secondary truncate font-['IBM_Plex_Sans']">
        {{ collection.name }}
      </span>
    </div>

    <!-- Children (when expanded) -->
    <div v-if="isExpanded && hasChildren">
      <!-- Child Collections -->
      <TreeItem
        v-for="child in childCollections"
        :key="child.id"
        :collection="child"
        :requests="getRequestsByCollection(child.id)"
        :child-collections="getChildCollections(child.id)"
        :all-collections="allCollections"
        :all-requests="allRequests"
        :expanded-nodes="expandedNodes"
        :level="level + 1"
        :selected-requests="selectedRequests"
        @toggle="(id) => emit('toggle', id)"
        @select-request="(req) => emit('selectRequest', req)"
        @request-select="(req, evt) => emit('request-select', req, evt)"
        @delete-collection="(id) => emit('delete-collection', id)"
        @rename-collection="(id) => emit('rename-collection', id)"
        @delete-request="(id) => emit('delete-request', id)"
        @delete-selected="() => emit('delete-selected')"
        @new-request="(id) => emit('new-request', id)"
      />

      <!-- Requests -->
      <div
        v-for="request in requests"
        :key="request.id"
        class="flex items-center gap-1 py-1.5 cursor-pointer transition-colors duration-100 request-item"
        :class="{ 'bg-surface-elevated': isRequestSelected(request.id), 'hover:bg-surface-elevated': !isRequestSelected(request.id) }"
        :style="{ paddingLeft: `${level * 16 + 8}px` }"
        @click.stop="handleRequestClick(request, $event)"
        @contextmenu="handleRequestContextMenu($event, request)" data-contextmenu
      >
        <span class="w-2 h-4 flex-shrink-0"></span>
        <span
          class="w-8 px-1 py-0.5 text-[10px] font-semibold rounded uppercase font-['JetBrains_Mono'] text-left flex-shrink-0"
          :class="getMethodColor(request.method)"
        >
          {{ getMethodAbbr(request.method) }}
        </span>
        <span class="text-xs leading-none text-text-secondary truncate font-['IBM_Plex_Sans']">
          {{ request.name }}
        </span>
      </div>
    </div>

    <!-- Context Menu -->
    <ContextMenu
      v-if="contextMenu.show"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :items="contextMenuItems"
      @close="closeContextMenu"
    />
    <!-- Request Context Menu -->
    <ContextMenu
      v-if="requestContextMenu.show"
      :x="requestContextMenu.x"
      :y="requestContextMenu.y"
      :items="requestContextMenuItems"
      @close="closeRequestContextMenu"
    />
  </div>
</template>
