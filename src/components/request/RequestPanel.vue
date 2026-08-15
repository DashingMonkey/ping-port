<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount, onUnmounted, watch } from "vue";
import { useI18n } from 'vue-i18n'
import { invoke } from "@tauri-apps/api/core";
import UrlBar from "./UrlBar.vue";
import ParamsTab from "./ParamsTab.vue";
import HeadersTab from "./HeadersTab.vue";
import BodyTab from "./BodyTab.vue";
import AuthTab from "./AuthTab.vue";
import ScriptsTab from "./ScriptsTab.vue";
import ResponsePanel, {
  type HttpResponse,
} from "../response/ResponsePanel.vue";
import { useEnvironmentsStore } from "../../stores/environments";
import { useTabsStore } from "../../stores/tabs";
import { useRequestsStore } from "../../stores/requests";
import { useCollectionsStore } from "../../stores/collections";
import { useSettingsStore } from "../../stores/settings";
import { replaceVariables } from "../../lib/variables";
import { toast } from "../../composables/useToast";
import type {
  RequestState,
  HttpMethod,
  KeyValuePair,
  RequestBody,
  AuthConfig,
} from "../../stores/types";
import type { TestResult } from "../response/TestResults.vue";

interface SendRequestResponse {
  success: boolean
  error: string | null
  status: number | null
  status_text: string | null
  headers: Record<string, string> | null
  body: string | null
  time_ms: number | null
  size_bytes: number | null
}

const props = defineProps<{
  initialState?: RequestState;
  initialTitle?: string;
  requestId?: string;
}>();

const defaultState: RequestState = {
  method: "GET",
  url: "",
  params: [],
  headers: [],
  body: { type: "none", content: "" },
  auth: { type: "none" },
  preRequestScript: "",
  testScript: "",
};

const requestState = reactive<RequestState>(
  props.initialState ?? { ...defaultState },
);

const { t } = useI18n()
const settingsStore = useSettingsStore();
const collectionsStore = useCollectionsStore();
const environmentsStore = useEnvironmentsStore();
const tabsStore = useTabsStore();
const requestsStore = useRequestsStore();

const response = ref<HttpResponse | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const activeTab = ref<"params" | "headers" | "body" | "auth" | "scripts">(
  tabsStore.activeTab ? tabsStore.getSubTab(tabsStore.activeTab.id) : "params",
);

// Remember the selected sub-tab so it survives top-level tab switches (remounts)
watch(activeTab, (val) => {
  const id = tabsStore.activeTab?.id;
  if (id) tabsStore.setSubTab(id, val);
});
const scriptVariables = ref<Record<string, string>>({});
const testResults = ref<TestResult[]>([]);
const consoleLogs = ref<string[]>([]);
const requestTitle = ref(props.initialTitle || "");

// Track current request ID (can be updated after saving new request)
const currentRequestId = ref(props.requestId);

// Track selected collection ID for saving
const selectedCollectionId = ref<string>('');

// Watch for requestId changes to sync collectionId
watch(
  () => props.requestId,
  (newRequestId) => {
    currentRequestId.value = newRequestId;
    if (newRequestId) {
      const request = requestsStore.requests.find(r => r.id === newRequestId);
      if (request?.collectionId) {
        selectedCollectionId.value = request.collectionId;
      }
    } else {
      // For new requests, clear collectionId to force user selection
      selectedCollectionId.value = '';
    }
  },
  { immediate: true }
);

// Compute effective variables for the current tab/collection context
const effectiveVariables = computed(() => {
  const activeTab = tabsStore.activeTab;
  const tabId = activeTab?.id ?? null;
  let collectionId: string | null = null;
  if (currentRequestId.value) {
    const request = requestsStore.requests.find(r => r.id === currentRequestId.value);
    collectionId = request?.collectionId ?? null;
  }
  return environmentsStore.getEffectiveVariables(tabId, collectionId);
});

// Track the draft ID at component mount time (before any tab switch)
let mountedDraftId: string | null = null;

// Get the draft for the current tab (for both new and existing requests)
const currentDraftId = computed(() => {
  const activeTab = tabsStore.activeTab;
  if (!activeTab) return null;
  const draft = tabsStore.drafts.find(d => d.tabId === activeTab.id);
  return draft?.id ?? null;
});

// Watch for changes in requestState and update draft in real-time
watch(
  requestState,
  (newState) => {
    if (currentDraftId.value) {
      tabsStore.updateDraftState(currentDraftId.value, JSON.parse(JSON.stringify(newState)));
      tabsStore.markDirty(currentDraftId.value);
    }
  },
  { deep: true },
);

// Watch for collectionId changes and mark dirty
watch(
  () => selectedCollectionId.value,
  () => {
    if (currentDraftId.value) {
      tabsStore.markDirty(currentDraftId.value);
    }
  }
);

// Watch for tab switches to save the previous tab's draft before switching
watch(
  () => tabsStore.activeTabId,
  () => {
    // Save the draft we captured at mount time (this is the tab that's about to be hidden)
    if (mountedDraftId) {
      tabsStore.updateDraftState(mountedDraftId, { ...requestState });
    }
    // Update mountedDraftId for the new active tab
    mountedDraftId = currentDraftId.value;

    // Sync collectionId for the new active tab
    const activeTab = tabsStore.activeTab;
    if (activeTab) {
      if (activeTab.requestId) {
        const request = requestsStore.requests.find(r => r.id === activeTab.requestId);
        selectedCollectionId.value = request?.collectionId || '';
      } else {
        // For new tabs, clear collectionId to force user selection
        selectedCollectionId.value = '';
      }
    }
  }
);

// Capture the draft ID when component mounts
// If no draft exists (e.g., restored from localStorage), create one
onMounted(() => {
  if (!currentDraftId.value) {
    const activeTab = tabsStore.activeTab;
    if (activeTab) {
      const draft = tabsStore.createDraft(activeTab.id, { ...requestState });
      mountedDraftId = draft.id;
      return;
    }
  }
  mountedDraftId = currentDraftId.value;
});

// Safety net: save draft before component unmounts
onBeforeUnmount(() => {
  if (mountedDraftId) {
    tabsStore.updateDraftState(mountedDraftId, { ...requestState });
  }
});



// Split bar state
const splitRatio = ref(settingsStore.splitRatio);
const isDragging = ref(false);
const containerRef = ref<HTMLElement | null>(null);
const startDragY = ref(0);
const startRatio = ref(55);

watch(
  () => settingsStore.splitRatio,
  (newVal) => {
    splitRatio.value = newVal;
  },
);

function startDrag(event: MouseEvent) {
  event.preventDefault();
  isDragging.value = true;
  startDragY.value = event.clientY;
  startRatio.value = splitRatio.value;
  document.addEventListener("mousemove", onDrag);
  document.addEventListener("mouseup", stopDrag);
}

function onDrag(event: MouseEvent) {
  if (!isDragging.value || !containerRef.value) return;
  event.preventDefault();
  const rect = containerRef.value.getBoundingClientRect();
  const delta = event.clientY - startDragY.value;
  const deltaRatio = (delta / rect.height) * 100;
  splitRatio.value = Math.min(80, Math.max(20, startRatio.value + deltaRatio));
}

function stopDrag() {
  isDragging.value = false;
  document.removeEventListener("mousemove", onDrag);
  document.removeEventListener("mouseup", stopDrag);
  settingsStore.setSplitRatio(splitRatio.value);
}

onUnmounted(() => {
  document.removeEventListener("mousemove", onDrag);
  document.removeEventListener("mouseup", stopDrag);
});

function buildRustAuth(auth: AuthConfig) {
  if (auth.type === "none") return null;
  const kind = auth.type === "api-key" ? "api-key" : auth.type;
  if (auth.type === "basic" && auth.basic) {
    return {
      kind,
      username: auth.basic.username,
      password: auth.basic.password,
    };
  }
  if (auth.type === "bearer" && auth.bearer) {
    return { kind, token: auth.bearer.token };
  }
  if (auth.type === "api-key" && auth.apiKey) {
    return {
      kind,
      key: auth.apiKey.key,
      value: auth.apiKey.value,
      in_header: auth.apiKey.in === "header",
    };
  }
  return null;
}

function keyValueArrayToObject(
  arr: { key: string; value: string; enabled?: boolean }[],
): Record<string, string> {
  return Object.fromEntries(
    arr
      .filter((p) => p.enabled !== false && p.key.trim() !== "")
      .map((p) => [p.key, p.value]),
  );
}

interface ReplaceResult {
  url: string
  params: KeyValuePair[]
  headers: KeyValuePair[]
  body: RequestBody
}

function applyVariableReplacement(
  variables: Record<string, string>,
  url: string,
  params: KeyValuePair[],
  headers: KeyValuePair[],
  body: RequestBody,
): ReplaceResult {
  let replacedBody =
    body.type !== "none"
      ? { ...body, content: replaceVariables(body.content, variables) }
      : body;

  if (
    replacedBody.type === "form-data" ||
    replacedBody.type === "x-www-form-urlencoded"
  ) {
    try {
      const items = JSON.parse(replacedBody.content);
      const filtered = items.filter(
        (p: { key: string; enabled?: boolean }) =>
          p.enabled !== false && p.key.trim() !== "",
      );
      replacedBody = { ...replacedBody, content: JSON.stringify(filtered) };
    } catch {
      // Invalid JSON, send as-is
    }
  }

  return {
    url: replaceVariables(url, variables),
    params: params
      .filter((p) => p.enabled)
      .map((p) => ({
        ...p,
        key: replaceVariables(p.key, variables),
        value: replaceVariables(p.value, variables),
      })),
    headers: headers
      .filter((h) => h.enabled)
      .map((h) => ({
        ...h,
        key: replaceVariables(h.key, variables),
        value: replaceVariables(h.value, variables),
      })),
    body: replacedBody,
  };
}

async function runPreRequestScript(
  script: string,
  variables: Record<string, string>,
  method: string,
  url: string,
  params: KeyValuePair[],
  headers: KeyValuePair[],
  body: RequestBody,
): Promise<{
  url: string
  headers: KeyValuePair[]
  body: RequestBody
  method: string | null
  returnedVariables: Record<string, string>
  logs: string[]
}> {
  const inputJson = JSON.stringify({
    script_type: "pre_request",
    script,
    request: { method, url, params, headers, body },
    variables,
  });
  const scriptResultJson = await invoke<string>("execute_script", {
    input: inputJson,
  });
  const scriptResult = JSON.parse(scriptResultJson);

  const result = {
    url,
    headers,
    body,
    method: null as string | null,
    returnedVariables: scriptResult.variables || ({} as Record<string, string>),
    logs: (scriptResult.console_logs || []) as string[],
  };

  if (scriptResult.modified_request) {
    const mod = scriptResult.modified_request;
    if (mod.method) result.method = mod.method;
    if (mod.url) result.url = mod.url;
    if (mod.headers) result.headers = mod.headers;
    if (mod.body) result.body = mod.body;
  }

  return result;
}

async function runTestScript(
  script: string,
  variables: Record<string, string>,
  method: string,
  url: string,
  headers: KeyValuePair[],
  body: RequestBody,
  responseStatus: number | null,
  responseHeaders: Record<string, string> | Record<string, string[]> | null,
  responseBody: string | null,
  responseTime: number | null,
): Promise<{
  testResults: TestResult[]
  logs: string[]
  returnedVariables: Record<string, string>
}> {
  const testInputJson = JSON.stringify({
    script_type: "test",
    script,
    request: { method, url, headers, body },
    response: {
      status: responseStatus,
      headers: responseHeaders,
      body: responseBody,
      time: responseTime,
    },
    variables,
  });
  const testResultJson = await invoke<string>("execute_script", {
    input: testInputJson,
  });
  const testResult = JSON.parse(testResultJson);

  return {
    testResults: testResult.test_results || [],
    logs: testResult.console_logs || [],
    returnedVariables: testResult.variables || {},
  };
}

async function sendRequest() {
  if (!requestState.url.trim()) {
    error.value = t('request.urlRequired');
    response.value = null;
    return;
  }

  loading.value = true;
  error.value = null;
  response.value = null;
  testResults.value = [];

  try {
    const variables = effectiveVariables.value;

    // Replace variables
    let { url: replacedUrl, params: replacedParams, headers: replacedHeaders, body: replacedBody } =
      applyVariableReplacement(
        variables,
        requestState.url,
        requestState.params,
        requestState.headers,
        requestState.body,
      );

    // Pre-request script
    if (requestState.preRequestScript) {
      try {
        const allVariables = { ...variables, ...scriptVariables.value };
        const preResult = await runPreRequestScript(
          requestState.preRequestScript,
          allVariables,
          requestState.method,
          replacedUrl,
          requestState.params,
          replacedHeaders,
          replacedBody,
        );
        if (preResult.method) requestState.method = preResult.method as HttpMethod;
        if (preResult.url) replacedUrl = preResult.url;
        if (preResult.headers) replacedHeaders = preResult.headers;
        if (preResult.body) replacedBody = preResult.body;
        scriptVariables.value = preResult.returnedVariables;
        consoleLogs.value = preResult.logs;
      } catch (e) {
        error.value = t('request.preRequestError', { message: e instanceof Error ? e.message : String(e) });
        return;
      }
    }

    // Send request
    const result = await invoke<SendRequestResponse>("send_request", {
      input: {
        method: requestState.method,
        url: replacedUrl,
        query_params: keyValueArrayToObject(replacedParams),
        headers: keyValueArrayToObject(replacedHeaders),
        body: replacedBody.type !== "none" ? replacedBody : null,
        auth: buildRustAuth(requestState.auth),
      },
    });

    if (
      !result ||
      result.success === false ||
      result.status === null ||
      result.status === undefined
    ) {
      error.value = result?.error || t('request.invalidUrl');
      response.value = null;
      return;
    }

    response.value = {
      status: result.status!,
      status_text: result.status_text || "",
      headers: result.headers || {},
      body: result.body || "",
      time_ms: result.time_ms || 0,
      size_bytes: result.size_bytes || 0,
    };

    // Test script
    if (requestState.testScript) {
      try {
        const allTestVariables = { ...variables, ...scriptVariables.value };
        const testResult = await runTestScript(
          requestState.testScript,
          allTestVariables,
          requestState.method,
          replacedUrl,
          replacedHeaders,
          replacedBody,
          result.status,
          result.headers,
          result.body,
          result.time_ms,
        );
        testResults.value = testResult.testResults;
        if (testResult.logs.length > 0) {
          consoleLogs.value = [...consoleLogs.value, ...testResult.logs];
        }
        if (testResult.returnedVariables) {
          scriptVariables.value = testResult.returnedVariables;
        }
      } catch (e) {
        testResults.value = [
          {
            name: t('response.testScriptError'),
            passed: false,
            error: e instanceof Error ? e.message : String(e),
          },
        ];
      }
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

function safeDecode(s: string): string {
  try {
    return decodeURIComponent(s);
  } catch {
    return s;
  }
}

function handleUrlUpdate(url: string) {
  const queryIndex = url.indexOf("?");
  if (queryIndex !== -1) {
    const baseUrl = url.substring(0, queryIndex);
    const queryString = url.substring(queryIndex + 1);

    // Parse query params
    const newParams = queryString.split("&").reduce((params, pair) => {
      const [key, ...valueParts] = pair.split("=");
      if (key) {
        params.push({
          key: safeDecode(key),
          value: safeDecode(valueParts.join("=")),
          enabled: true,
        });
      }
      return params;
    }, [] as KeyValuePair[]);

    // Merge with existing params (avoid duplicates by key)
    const existingKeys = new Set(requestState.params.map((p) => p.key));
    const uniqueNewParams = newParams.filter((p) => !existingKeys.has(p.key));
    requestState.params = [...requestState.params, ...uniqueNewParams];

    requestState.url = baseUrl;
  } else {
    requestState.url = url;
  }
}

function handleMethodUpdate(method: HttpMethod) {
  requestState.method = method;
}

function handleParamsUpdate(params: KeyValuePair[]) {
  requestState.params = params;
}

function handleHeadersUpdate(headers: KeyValuePair[]) {
  requestState.headers = headers;
}

function handleBodyUpdate(body: RequestBody) {
  requestState.body = body;
}

function handleAuthUpdate(auth: AuthConfig) {
  requestState.auth = auth;
}

function handleScriptsUpdate(scripts: {
  preRequestScript: string;
  testScript: string;
}) {
  requestState.preRequestScript = scripts.preRequestScript;
  requestState.testScript = scripts.testScript;
}

function handleTitleUpdate(title: string) {
  requestTitle.value = title;
}

function handleSave() {
  if (currentRequestId.value) {
    saveExistingRequest();
  } else {
    saveNewRequest();
  }
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
    handleSave();
  }
}

async function saveNewRequest() {
  if (!selectedCollectionId.value) {
    toast.error(t('request.selectCollection'));
    return;
  }
  if (!requestTitle.value.trim()) {
    toast.error(t('request.nameRequired'));
    return;
  }

  try {
    const created = await requestsStore.createRequest({
      collectionId: selectedCollectionId.value,
      name: requestTitle.value.trim(),
      method: requestState.method,
      url: requestState.url,
      params: JSON.stringify(requestState.params),
      headers: JSON.stringify(requestState.headers),
      body: JSON.stringify(requestState.body),
      auth: JSON.stringify(requestState.auth),
      preRequestScript: requestState.preRequestScript,
      testScript: requestState.testScript,
    });

    currentRequestId.value = created.id;

    // Update tab title and bind tab to request
    const activeTab = tabsStore.activeTab;
    if (activeTab) {
      tabsStore.updateTabTitle(activeTab.id, `${requestState.method} ${requestTitle.value.trim()}`);
      tabsStore.bindTabToRequest(activeTab.id, created.id, requestState.method);
    }

    // Mark draft as clean after saving
    if (currentDraftId.value) {
      tabsStore.markClean(currentDraftId.value);
    }

    // Expand the collection in sidebar so the user can see the saved request
    collectionsStore.expandToCollection(selectedCollectionId.value);

    // Select the newly created request in sidebar
    collectionsStore.selectedRequests = new Set([created.id]);

    toast.success(t('request.saved'), 1000);
  } catch (e) {
    toast.error(t('request.saveFailed', { error: e instanceof Error ? e.message : String(e) }));
  }
}

async function saveExistingRequest() {
  if (!currentRequestId.value) return;

  if (!selectedCollectionId.value) {
    toast.error(t('request.selectCollection'));
    return;
  }

  const existingRequest = requestsStore.requests.find(r => r.id === currentRequestId.value);
  if (!existingRequest) return;

  const oldCollectionId = existingRequest.collectionId;
  const collectionChanged = oldCollectionId !== selectedCollectionId.value;

  await requestsStore.updateRequest({
    id: currentRequestId.value,
    collectionId: selectedCollectionId.value,
    name: requestTitle.value || t('request.untitled'),
    method: requestState.method,
    url: requestState.url,
    params: JSON.stringify(requestState.params),
    headers: JSON.stringify(requestState.headers),
    body: JSON.stringify(requestState.body),
    auth: JSON.stringify(requestState.auth),
    preRequestScript: requestState.preRequestScript,
    testScript: requestState.testScript,
  });

  // Mark draft as clean after saving
  if (currentDraftId.value) {
    tabsStore.markClean(currentDraftId.value);
  }

  // Expand the new collection if it changed
  if (collectionChanged) {
    collectionsStore.expandToCollection(selectedCollectionId.value);
  }

  toast.success(t('request.saved'), 1000);
}

onMounted(async () => {
  document.addEventListener('keydown', handleKeydown);

  // Ensure collections are loaded
  await collectionsStore.fetchCollections();

  // For existing requests, use the request's collection
  if (currentRequestId.value) {
    const request = requestsStore.requests.find(r => r.id === currentRequestId.value);
    if (request?.collectionId) {
      selectedCollectionId.value = request.collectionId;
    }
  }
  // For new requests, leave selectedCollectionId empty to force user selection
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
});

</script>

<template>
  <div ref="containerRef" class="flex flex-col h-full">
    <!-- URL Bar -->
    <div
      class="p-1.5 border-b border-border-default bg-surface-base flex-shrink-0"
    >
      <UrlBar
        :model-value="requestState.url"
        :method="requestState.method"
        :title="requestTitle"
        :collection-id="selectedCollectionId"
        :collections="collectionsStore.collections"
        :loading="loading"
        @update:model-value="handleUrlUpdate"
        @update:method="handleMethodUpdate"
        @update:title="handleTitleUpdate"
        @update:collection-id="selectedCollectionId = $event"
        @send="sendRequest"
        @save="handleSave"
      />
    </div>

    <!-- Tabs -->
    <div class="border-b border-border-default bg-surface-base flex-shrink-0">
      <div class="flex px-1">
        <button
          v-for="tab in (['params', 'auth', 'headers', 'body', 'scripts'] as const)"
          :key="tab"
          type="button"
          @click="activeTab = tab as typeof activeTab"
          class="px-3 py-1.5 text-xs font-medium border-b-2 transition-colors duration-150 focus:outline-none"
          :class="
            activeTab === tab
              ? 'border-accent text-accent'
              : 'border-transparent text-text-muted hover:text-text-secondary'
          "
        >
          {{ { params: t('tabs.params'), auth: t('tabs.auth'), headers: t('tabs.headers'), body: t('tabs.body'), scripts: t('tabs.scripts') }[tab] }}
        </button>
      </div>
    </div>

    <!-- Request Panel (resizable) -->
    <div class="overflow-auto" :style="{ height: `${splitRatio}%` }">
      <ParamsTab
        v-if="activeTab === 'params'"
        :model-value="requestState.params"
        :variables="effectiveVariables"
        @update:model-value="handleParamsUpdate"
      />
      <HeadersTab
        v-else-if="activeTab === 'headers'"
        :model-value="requestState.headers"
        :variables="effectiveVariables"
        @update:model-value="handleHeadersUpdate"
      />
      <BodyTab
        v-else-if="activeTab === 'body'"
        :model-value="requestState.body"
        :variables="effectiveVariables"
        @update:model-value="handleBodyUpdate"
      />
      <AuthTab
        v-else-if="activeTab === 'auth'"
        :model-value="requestState.auth"
        @update:model-value="handleAuthUpdate"
      />
      <ScriptsTab
        v-else-if="activeTab === 'scripts'"
        :model-value="{
          preRequestScript: requestState.preRequestScript,
          testScript: requestState.testScript,
        }"
        @update:model-value="handleScriptsUpdate"
      />
    </div>

    <!-- Resizable Split Bar -->
    <div
      class="relative z-10 select-none cursor-row-resize"
      style="height: 1px; background: transparent"
      @mousedown="startDrag"
    >
      <!-- Visual line (very thin) -->
      <div
        class="absolute left-0 right-0 top-0 h-px transition-all duration-150"
        :style="
          isDragging
            ? 'background-color: rgba(0, 122, 204, 0.5); height: 2px'
            : 'background-color: var(--border-default)'
        "
      />

      <!-- Hover zone (transparent 20px tall area) -->
      <div
        class="absolute -top-2 -bottom-2 left-0 right-0"
        :class="{ 'cursor-row-resize': isDragging }"
      />
    </div>

    <!-- Response Panel (resizable) -->
    <div
      class="border-t border-border-default overflow-hidden flex-shrink-0"
      :style="{ height: `${100 - splitRatio}%` }"
    >
      <ResponsePanel
        :response="response"
        :loading="loading"
        :test-results="testResults"
        :console-logs="consoleLogs"
        :error="error"
      />
    </div>

  </div>
</template>
