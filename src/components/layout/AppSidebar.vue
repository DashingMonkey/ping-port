<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import CollectionTree from '../sidebar/CollectionTree.vue'
import { useSettingsStore } from '../../stores/settings'
import { useCollectionsStore } from '../../stores/collections'
import { prompt } from '../../composables/usePrompt'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const collectionsStore = useCollectionsStore()

const handleCreateCollection = async () => {
  const name = await prompt(t('collection.enterCollectionName'))
  if (name) {
    await collectionsStore.createCollection({
      name,
      parentId: undefined,
    })
  }
}
</script>

<template>
  <aside class="h-full flex flex-col bg-surface-deep">
    <!-- Collections Panel -->
    <template v-if="settingsStore.activeActivity === 'collections'">
      <div class="px-3 py-2.5 border-b border-border-default flex items-center justify-between">
        <div class="text-xs font-semibold tracking-[0.08em] text-text-secondary uppercase">
          {{ t('collection.collections') }}
        </div>
        <button
          @click="handleCreateCollection"
          class="p-1 text-text-muted hover:text-accent rounded-md hover:bg-surface-elevated transition-colors duration-150"
          :title="t('collection.newCollection')"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>
      <div class="flex-1 overflow-auto">
        <CollectionTree />
      </div>
    </template>
  </aside>
</template>
