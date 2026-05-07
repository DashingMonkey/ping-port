<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = withDefaults(defineProps<{
  data: unknown
  depth?: number
  keyName?: string | number | null
}>(), {
  depth: 0,
  keyName: null
})

const expanded = ref(props.depth === 0)

const contextMenu = ref({
  show: false,
  x: 0,
  y: 0,
  nodeData: null as unknown,
  nodeKey: null as string | number | null,
})

function handleContextMenu(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  contextMenu.value = {
    show: true,
    x: e.clientX,
    y: e.clientY,
    nodeData: props.data,
    nodeKey: props.keyName,
  }
}

function handleChildContextMenu(e: MouseEvent, key: string | number, val: unknown) {
  e.preventDefault()
  e.stopPropagation()
  contextMenu.value = {
    show: true,
    x: e.clientX,
    y: e.clientY,
    nodeData: val,
    nodeKey: key,
  }
}

function closeMenu() {
  contextMenu.value.show = false
}

const isExpandable = computed(() => {
  return props.data !== null && typeof props.data === 'object'
})

const typeLabel = computed(() => {
  if (Array.isArray(props.data)) {
    return 'Array[' + props.data.length + ']'
  }
  if (props.data !== null && typeof props.data === 'object') {
    return 'Object{' + Object.keys(props.data).length + '}'
  }
  return ''
})

const collapsedLabel = computed(() => {
  return Array.isArray(props.data) ? '[...]' : '{...}'
})

const entries = computed(() => {
  if (Array.isArray(props.data)) {
    return props.data.map((v, i) => [i, v] as [number, any])
  }
  if (props.data && typeof props.data === 'object') {
    return Object.entries(props.data)
  }
  return []
})

function toggle() {
  expanded.value = !expanded.value
}

function formatValue(val: unknown): string {
  if (val === null) return 'null'
  if (typeof val === 'string') return '"' + val + '"'
  if (typeof val === 'boolean') return val ? 'true' : 'false'
  return String(val)
}

function getValueClass(val: unknown): string {
  if (val === null) return 'text-slate-600 dark:text-slate-400'
  if (typeof val === 'string') return 'text-[#368422] dark:text-emerald-400'
  if (typeof val === 'number') return 'text-sky-600 dark:text-sky-400'
  if (typeof val === 'boolean') return 'text-amber-600 dark:text-amber-400'
  return 'text-inherit'
}

async function copyAsJson() {
  const data = contextMenu.value.nodeKey !== null
    ? { [contextMenu.value.nodeKey]: contextMenu.value.nodeData }
    : contextMenu.value.nodeData
  await navigator.clipboard.writeText(JSON.stringify(data, null, 2))
  closeMenu()
}
</script>

<template>
  <div class="font-mono text-[13px]" :style="{ paddingLeft: depth > 0 ? '16px' : '0' }">
    <div
      v-if="isExpandable"
      class="flex items-center gap-1 py-px cursor-pointer hover:bg-surface-elevated rounded"
      @click="toggle"
      @contextmenu="handleContextMenu"
    >
      <svg
        class="w-3.5 h-3.5 text-text-muted transition-transform duration-100 flex-shrink-0"
        :class="{ 'rotate-90': expanded }"
        fill="none" viewBox="0 0 24 24" stroke="currentColor"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
      </svg>
      <span v-if="keyName !== null" class="text-[#347896] dark:text-cyan-400">{{ keyName }}</span>
      <span v-if="keyName !== null" class="text-slate-500 mx-1">:</span>
      <span class="text-warning text-[11px]">{{ typeLabel }}</span>
      <span v-if="!expanded" class="text-text-muted">{{ collapsedLabel }}</span>
    </div>

    <div v-else class="flex items-center gap-1 py-px cursor-pointer hover:bg-surface-elevated rounded" @contextmenu="handleContextMenu">
      <span class="w-3.5"></span>
      <span v-if="keyName !== null" class="text-[#347896] dark:text-cyan-400">{{ keyName }}</span>
      <span v-if="keyName !== null" class="text-slate-500 mx-1">:</span>
      <span :class="getValueClass(data)">{{ formatValue(data) }}</span>
    </div>

    <div v-if="isExpandable && expanded">
      <div v-for="[key, val] in entries" :key="key">
        <JsonTreeView
          v-if="val !== null && typeof val === 'object'"
          :data="val"
          :key-name="key"
          :depth="depth + 1"
        />
        <div v-else class="flex items-center gap-1 py-px cursor-pointer hover:bg-surface-elevated rounded" :style="{ paddingLeft: '16px' }" @contextmenu="(e) => handleChildContextMenu(e, key, val)">
          <span class="w-3.5"></span>
          <span class="text-[#347896] dark:text-cyan-400">{{ key }}</span>
          <span class="text-slate-500 mx-1">:</span>
          <span :class="getValueClass(val)">{{ formatValue(val) }}</span>
        </div>
      </div>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenu.show"
        class="fixed z-50 bg-surface-base border border-border-default rounded-lg shadow-lg py-1 min-w-[160px]"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
        @click.stop
      >
        <button
          class="w-full px-4 py-2 text-left text-sm text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="copyAsJson"
        >
          {{ t('response.copyAsJson') }}
        </button>
      </div>
      <div
        v-if="contextMenu.show"
        class="fixed inset-0 z-40"
        @click="closeMenu"
        @contextmenu.prevent="closeMenu"
      ></div>
    </Teleport>
  </div>
</template>
