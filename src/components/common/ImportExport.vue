<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCollectionsStore } from '../../stores/collections'
import { useRequestsStore } from '../../stores/requests'

const { t } = useI18n()
import { toPostmanCollection, fromPostmanCollection, exportToJson, importFromJson, type PostmanCollection } from '../../lib/import-export'

defineProps<{
  mode: 'import' | 'export'
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const collectionsStore = useCollectionsStore()
const requestsStore = useRequestsStore()

const importText = ref('')
const importError = ref('')
const exportText = ref('')

const handleExport = () => {
  const postmanData = toPostmanCollection(
    collectionsStore.collections,
    requestsStore.requests
  )
  exportText.value = exportToJson(postmanData)
}

const handleImport = async () => {
  importError.value = ''
  try {
    const data = importFromJson(importText.value) as PostmanCollection
    const { collections, requests } = fromPostmanCollection(data)

    for (const col of collections) {
      await collectionsStore.createCollection({
        name: col.name,
      })
    }

    for (const req of requests) {
      await requestsStore.createRequest({
        name: req.name,
        method: req.method,
        url: req.url,
        collectionId: req.collectionId,
      })
    }

    emit('close')
  } catch (e) {
    importError.value = t('importExport.invalidJson', { error: `${e}` })
  }
}

const downloadExport = () => {
  const blob = new Blob([exportText.value], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = t('importExport.filename')
  a.click()
  URL.revokeObjectURL(url)
}

const copyToClipboard = async () => {
  await navigator.clipboard.writeText(exportText.value)
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
    <div class="bg-surface-base rounded-lg shadow-xl w-full max-w-lg mx-4 border border-border-default">
      <div class="px-6 py-4 border-b border-border-default">
        <h3 class="text-lg font-semibold text-text-primary font-['IBM_Plex_Sans']">{{ t(mode === 'import' ? 'importExport.importTitle' : 'importExport.exportTitle') }}</h3>
      </div>

      <div class="px-6 py-4">
        <div v-if="mode === 'import'">
          <p class="text-sm text-text-secondary mb-4 font-['IBM_Plex_Sans']">
            {{ t('importExport.importInstruction') }}
          </p>
          <textarea
            v-model="importText"
            rows="10"
            :placeholder="t('importExport.placeholder')"
            class="w-full px-3 py-2 text-sm font-mono bg-surface-deep border border-border-default rounded focus:outline-none text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
          />
          <p v-if="importError" class="text-sm text-error mt-2 font-['IBM_Plex_Sans']">{{ importError }}</p>
        </div>

        <div v-else>
          <p class="text-sm text-text-secondary mb-4 font-['IBM_Plex_Sans']">
            {{ t('importExport.exportInstruction') }}
          </p>
          <textarea
            v-model="exportText"
            rows="10"
            readonly
            class="w-full px-3 py-2 text-sm font-mono bg-surface-deep border border-border-default rounded focus:outline-none text-text-secondary font-['JetBrains_Mono']"
          />
          <div class="flex gap-2 mt-4">
            <button
              @click="downloadExport"
              class="px-4 py-2 text-sm bg-accent hover:bg-cyan-400 text-surface-deep rounded font-semibold transition-colors duration-150 font-['IBM_Plex_Sans']"
            >
              {{ t('importExport.download') }}
            </button>
            <button
              @click="copyToClipboard"
              class="px-4 py-2 text-sm bg-surface-elevated hover:bg-border-default text-text-secondary rounded transition-colors duration-150 font-['IBM_Plex_Sans']"
            >
              {{ t('importExport.copyClipboard') }}
            </button>
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-border-default flex justify-end gap-3">
        <button
          @click="emit('close')"
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-surface-elevated rounded transition-colors duration-150 font-['IBM_Plex_Sans']"
        >
          {{ t('importExport.cancel') }}
        </button>
        <button
          v-if="mode === 'import'"
          @click="handleImport"
          class="px-4 py-2 text-sm bg-accent hover:bg-cyan-400 text-surface-deep rounded font-semibold transition-colors duration-150 font-['IBM_Plex_Sans']"
        >
          {{ t('importExport.import') }}
        </button>
        <button
          v-else
          @click="handleExport"
          class="px-4 py-2 text-sm bg-accent hover:bg-cyan-400 text-surface-deep rounded font-semibold transition-colors duration-150 font-['IBM_Plex_Sans']"
        >
          {{ t('importExport.generateExport') }}
        </button>
      </div>
    </div>
  </div>
</template>
