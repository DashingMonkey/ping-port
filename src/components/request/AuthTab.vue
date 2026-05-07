<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { AuthConfig } from '../../stores/types'

const { t } = useI18n()

const props = defineProps<{
  modelValue: AuthConfig
}>()

const emit = defineEmits<{
  'update:modelValue': [auth: AuthConfig]
}>()

const authTypes = [
  { value: 'none', label: 'No Auth' },
  { value: 'basic', label: 'Basic Auth' },
  { value: 'bearer', label: 'Bearer Token' },
  { value: 'api-key', label: 'API Key' }
] as const

const authLabelMap = computed(() => ({
  none: t('auth.noAuth'),
  basic: t('auth.basicAuth'),
  bearer: t('auth.bearerToken'),
  'api-key': t('auth.apiKey'),
}))

function handleTypeChange(type: AuthConfig['type']) {
  const newAuth: AuthConfig = { type }
  if (type === 'basic') {
    newAuth.basic = { username: '', password: '' }
  } else if (type === 'bearer') {
    newAuth.bearer = { token: '' }
  } else if (type === 'api-key') {
    newAuth.apiKey = { key: '', value: '', in: 'header' }
  }
  emit('update:modelValue', newAuth)
}

function updateBasic(field: 'username' | 'password', value: string) {
  const basic = props.modelValue.basic ?? { username: '', password: '' }
  emit('update:modelValue', {
    ...props.modelValue,
    basic: { ...basic, [field]: value }
  })
}

function updateBearer(field: 'token', value: string) {
  const bearer = props.modelValue.bearer ?? { token: '' }
  emit('update:modelValue', {
    ...props.modelValue,
    bearer: { ...bearer, [field]: value }
  })
}

function updateApiKey(field: 'key' | 'value' | 'in', value: string) {
  const apiKey = props.modelValue.apiKey ?? { key: '', value: '', in: 'header' as const }
  emit('update:modelValue', {
    ...props.modelValue,
    apiKey: { ...apiKey, [field]: value }
  })
}
</script>

<template>
  <div class="p-2 space-y-2">
    <div>
      <label class="block text-xs font-medium text-text-secondary mb-1">
        {{ t('auth.authType') }}
      </label>
      <select
        :value="modelValue.type"
        @change="handleTypeChange(($event.target as HTMLSelectElement).value as AuthConfig['type'])"
        class="block w-full px-2 py-1 text-xs rounded border border-border-default bg-surface-deep focus:outline-none focus:border-accent text-text-primary font-['IBM_Plex_Sans']"
      >
        <option v-for="at in authTypes" :key="at.value" :value="at.value">
          {{ authLabelMap[at.value] }}
        </option>
      </select>
    </div>

    <div v-if="modelValue.type === 'basic'" class="space-y-2">
      <div>
        <label class="block text-xs font-medium text-text-secondary mb-1">
          {{ t('auth.username') }}
        </label>
        <input
          type="text"
          :value="modelValue.basic?.username"
          @input="updateBasic('username', ($event.target as HTMLInputElement).value)"
          :placeholder="t('auth.username')"
          class="block w-full px-2 py-1 text-xs rounded border border-border-default bg-surface-deep focus:outline-none focus:border-accent text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
        />
      </div>
      <div>
        <label class="block text-xs font-medium text-text-secondary mb-1">
          {{ t('auth.password') }}
        </label>
        <input
          type="password"
          :value="modelValue.basic?.password"
          @input="updateBasic('password', ($event.target as HTMLInputElement).value)"
          :placeholder="t('auth.password')"
          class="block w-full px-2 py-1 text-xs rounded border border-border-default bg-surface-deep focus:outline-none focus:border-accent text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
        />
      </div>
    </div>

    <div v-if="modelValue.type === 'bearer'" class="space-y-2">
      <div>
        <label class="block text-xs font-medium text-text-secondary mb-1">
          {{ t('auth.token') }}
        </label>
        <input
          type="text"
          :value="modelValue.bearer?.token"
          @input="updateBearer('token', ($event.target as HTMLInputElement).value)"
          :placeholder="t('auth.bearerTokenPlaceholder')"
          class="block w-full px-2 py-1 text-xs rounded border border-border-default bg-surface-deep focus:outline-none focus:border-accent text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
        />
      </div>
    </div>

    <div v-if="modelValue.type === 'api-key'" class="space-y-2">
      <div>
        <label class="block text-xs font-medium text-text-secondary mb-1">
          {{ t('auth.key') }}
        </label>
        <input
          type="text"
          :value="modelValue.apiKey?.key"
          @input="updateApiKey('key', ($event.target as HTMLInputElement).value)"
          :placeholder="t('auth.apiKeyName')"
          class="block w-full px-2 py-1 text-xs rounded border border-border-default bg-surface-deep focus:outline-none focus:border-accent text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
        />
      </div>
      <div>
        <label class="block text-xs font-medium text-text-secondary mb-1">
          {{ t('auth.value') }}
        </label>
        <input
          type="text"
          :value="modelValue.apiKey?.value"
          @input="updateApiKey('value', ($event.target as HTMLInputElement).value)"
          :placeholder="t('auth.apiKeyValue')"
          class="block w-full px-2 py-1 text-xs rounded border border-border-default bg-surface-deep focus:outline-none focus:border-accent text-text-primary placeholder-text-muted font-['JetBrains_Mono']"
        />
      </div>
      <div>
        <label class="block text-xs font-medium text-text-secondary mb-1">
          {{ t('auth.addTo') }}
        </label>
        <select
          :value="modelValue.apiKey?.in"
          @change="updateApiKey('in', ($event.target as HTMLSelectElement).value)"
          class="block w-full px-2 py-1 text-xs rounded border border-border-default bg-surface-deep focus:outline-none focus:border-accent text-text-primary font-['IBM_Plex_Sans']"
        >
          <option value="header">{{ t('auth.header') }}</option>
          <option value="query">{{ t('auth.queryParam') }}</option>
        </select>
      </div>
    </div>
  </div>
</template>
