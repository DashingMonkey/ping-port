import assert from 'node:assert/strict'
import { test } from 'node:test'
import { registerHooks } from 'node:module'
import { readFileSync, existsSync } from 'node:fs'
import { fileURLToPath, pathToFileURL } from 'node:url'
import ts from 'typescript'
import { parse, compileScript } from 'vue/compiler-sfc'

// Run the real TS modules and Vue setup functions without adding a test runtime.
registerHooks({
  resolve(specifier, context, nextResolve) {
    try { return nextResolve(specifier, context) } catch (error) {
      if (specifier.startsWith('.') && context.parentURL?.startsWith('file:')) {
        const base = fileURLToPath(new URL(specifier, context.parentURL))
        for (const suffix of ['.ts', '/index.ts']) {
          if (existsSync(base + suffix)) return { url: pathToFileURL(base + suffix).href, shortCircuit: true }
        }
      }
      throw error
    }
  },
  load(url, context, nextLoad) {
    if (url.startsWith('file:') && /\.(ts|vue)$/.test(url) && !url.includes('/node_modules/')) {
      let source = readFileSync(fileURLToPath(url), 'utf8')
      if (url.endsWith('.vue')) source = compileScript(parse(source).descriptor, { id: url }).content
      return { format: 'module', shortCircuit: true, source: ts.transpileModule(source, {
        compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 },
      }).outputText }
    }
    return nextLoad(url, context)
  },
})

const storage = new Map()
Object.defineProperty(globalThis, 'localStorage', { value: {
  getItem: key => storage.get(key) ?? null,
  setItem: (key, value) => storage.set(key, String(value)),
  removeItem: key => storage.delete(key),
  clear: () => storage.clear(),
} })
globalThis.window = globalThis
globalThis.addEventListener = () => {}
globalThis.removeEventListener = () => {}
globalThis.document = { body: { addEventListener() {} }, createElement: () => ({}), addEventListener() {}, removeEventListener() {} }

const { createPinia, setActivePinia } = await import('pinia')
const { nextTick, createRenderer } = await import('vue')
const { useTabsStore } = await import('../src/stores/tabs.ts')
const { useEnvironmentsStore } = await import('../src/stores/environments.ts')
const { DEFAULT_REQUEST_STATE } = await import('../src/stores/types.ts')
const { highlightVariables } = await import('../src/lib/highlight-variables.ts')
const { fromPostmanCollection, toPostmanCollection } = await import('../src/lib/import-export.ts')
const { default: i18n } = await import('../src/i18n.ts')
const { default: RequestPanel } = await import('../src/components/request/RequestPanel.vue')

function freshPinia() {
  localStorage.clear()
  const pinia = createPinia()
  setActivePinia(pinia)
  return pinia
}

const renderer = createRenderer({
  createElement: () => ({}), createText: () => ({}), createComment: () => ({}),
  setText() {}, setElementText() {}, parentNode: () => null, nextSibling: () => null,
  insert() {}, remove() {}, patchProp() {},
})

function mountPanel(initialState, pinia) {
  const app = renderer.createApp({ ...RequestPanel, render: () => null }, { initialState })
  app.use(pinia).use(i18n)
  const instance = app.mount({})
  return { panel: instance.$.setupState, unmount: () => app.unmount() }
}

test('variable highlights escape attributes once and look up raw names', () => {
  const name = 'x" onmouseover="alert(1)'
  const html = highlightVariables(`prefix <script> {{${name}}} {{a&b}}`, { [name]: 'yes', 'a&b': 'yes' })
  assert.ok(!html.includes('<script>'))
  assert.ok(!html.includes('data-varname="x"'))
  assert.ok(html.includes('data-varname="x&quot; onmouseover=&quot;alert(1)"'))
  assert.ok(html.includes('data-varname="a&amp;b"'))
  assert.ok(!html.includes('&amp;amp;'))
  assert.equal((html.match(/class="var-found"/g) || []).length, 2)
})

test('workspace loading restores, isolates, flushes and renames drafts', async (t) => {
  freshPinia()
  localStorage.setItem('pingport-last-workspace', 'a')
  const savedTab = { id: 'saved', requestId: '', method: 'GET', title: 'saved', position: 1 }
  localStorage.setItem('pingport-a-tabs', JSON.stringify({ tabs: [savedTab], activeTabId: 'saved' }))
  const tabs = useTabsStore()
  tabs.loadTabs()
  assert.equal(tabs.activeTabId, 'saved')
  tabs.createDraft('saved', structuredClone(DEFAULT_REQUEST_STATE))
  tabs.setSubTab('saved', 'body')
  await nextTick()
  localStorage.setItem('pingport-last-workspace', 'b')
  tabs.loadTabs()
  assert.equal(tabs.tabs.length, 0)
  assert.equal(tabs.drafts.length, 0)
  assert.equal(tabs.activeTabId, null)
  assert.equal(tabs.getSubTab('saved'), 'params')
  assert.equal(JSON.parse(localStorage.getItem('pingport-a-drafts')).length, 1)
  await nextTick()
  tabs.persist()
  assert.deepEqual(JSON.parse(localStorage.getItem('pingport-b-drafts')), [])
  localStorage.setItem('pingport-last-workspace', 'a')
  tabs.loadTabs()
  assert.equal(tabs.drafts.length, 1)
  tabs.renameWorkspaceStorage('a', 'renamed')
  localStorage.setItem('pingport-last-workspace', 'renamed')
  await nextTick()
  tabs.persist()
  assert.equal(localStorage.getItem('pingport-a-drafts'), null)
  assert.equal(JSON.parse(localStorage.getItem('pingport-renamed-drafts')).length, 1)
  const warning = t.mock.method(console, 'warn', () => {})
  localStorage.setItem('pingport-broken-tabs', '{')
  localStorage.setItem('pingport-last-workspace', 'broken')
  tabs.loadTabs()
  assert.equal(tabs.tabs.length, 0)
  assert.equal(tabs.drafts.length, 0)
  assert.equal(warning.mock.callCount(), 1)
  await nextTick()
  tabs.persist()
})

test('Postman import/export preserves request fields, auth, scripts, files and disabled entries', () => {
  const data = {
    info: { name: 'Review', schema: 'https://schema.getpostman.com/json/collection/v2.1.0/collection.json' },
    item: [
      { name: 'empty folder', item: [] },
      { name: 'basic', request: {
        method: 'POST', url: { raw: 'https://example.test?a=1', query: [{ key: 'a', value: '2' }, { key: 'off', value: '3', disabled: true }] },
        header: [{ key: 'X-Off', value: 'off', disabled: true }],
        auth: { type: 'basic', basic: [{ key: 'password', value: 'secret' }, { key: 'username', value: 'user' }] },
        body: { mode: 'urlencoded', urlencoded: [{ key: 'field', value: 'a&b' }] },
      }, event: [{ listen: 'prerequest', script: { exec: ['pp.variables.set("token", "value");'] } }, { listen: 'test', script: { exec: ['console.log("done");'] } }] },
      { name: 'file', request: {
        url: 'https://example.test?x=hello+world',
        auth: { type: 'apikey', apikey: [{ key: 'in', value: 'query' }, { key: 'value', value: 'secret' }, { key: 'key', value: 'token' }] },
        body: { mode: 'formdata', formdata: [{ key: 'file', type: 'file', src: 'C:/upload.txt' }] },
      } },
    ],
  }
  const imported = fromPostmanCollection(data)
  assert.equal(imported.collections.length, 2)
  assert.equal(imported.requests[0].url, 'https://example.test')
  assert.equal(JSON.parse(imported.requests[0].auth).basic.username, 'user')
  const exported = toPostmanCollection(imported.collections, imported.requests)
  const items = exported.item[0].item
  const basic = items.find(item => item.name === 'basic')
  assert.equal(basic.request.body.mode, 'urlencoded')
  assert.ok(basic.request.body.urlencoded)
  assert.equal(basic.request.body.formdata, undefined)
  assert.equal(basic.request.url.raw, 'https://example.test?a=2')
  assert.equal(basic.request.url.query[1].disabled, true)
  assert.equal(basic.request.header[0].disabled, true)
  assert.equal(basic.event.length, 2)
  const file = items.find(item => item.name === 'file')
  assert.equal(file.request.body.formdata[0].src, 'C:/upload.txt')
  const roundtrip = fromPostmanCollection(exported)
  for (const original of imported.requests) {
    const restored = roundtrip.requests.find(r => r.name === original.name)
    for (const field of ['url', 'params', 'headers', 'body', 'auth', 'preRequestScript', 'testScript']) {
      assert.equal(restored[field], original[field], `${original.name}: ${field}`)
    }
  }
  assert.throws(() => fromPostmanCollection({}), /Expected a Postman/)
})

test('request panel applies script changes before substitution and sends enabled headers/body/params', async () => {
  const pinia = freshPinia()
  const tabs = useTabsStore()
  tabs.loadTabs()
  tabs.createTab('', 'POST', 'test')
  let sent
  let testInput
  globalThis.__TAURI_INTERNALS__ = { invoke: async (command, args) => {
    if (command === 'get_collections') return []
    if (command === 'execute_script') {
      const input = JSON.parse(args.input)
      if (input.script_type === 'test') {
        testInput = input
        return JSON.stringify({ variables: { next: 'ready' }, test_results: [], console_logs: [] })
      }
      assert.equal(input.request.body, 'before')
      assert.equal(input.request.url, 'https://example.test/{{token}}')
      assert.deepEqual(input.request.headers, { 'X-Keep': 'yes' })
      return JSON.stringify({
        variables: { token: 'fresh' }, console_logs: [],
        modified_request: { ...input.request, method: 'PUT', body: '', headers: { 'X-Token': '{{token}}' }, query_params: { added: '{{token}}' } },
      })
    }
    if (command === 'send_request') {
      sent = args.input
      return { success: true, status: 200, status_text: 'OK', headers: { 'set-cookie': ['a=1', 'b=2'] }, body: '{}', time_ms: 1 }
    }
    throw new Error(command)
  } }
  const original = {
    ...structuredClone(DEFAULT_REQUEST_STATE), method: 'POST', url: 'https://example.test/{{token}}',
    params: [{ key: 'old', value: 'old', enabled: true }],
    headers: [{ key: 'X-Keep', value: 'yes', enabled: true }, { key: 'X-Off', value: 'no', enabled: false }],
    body: { type: 'json', content: 'before' }, auth: { type: 'bearer', bearer: { token: '{{token}}' } },
    preRequestScript: 'pre', testScript: 'test',
  }
  const { panel, unmount } = mountPanel(original, pinia)
  await panel.sendRequest()
  assert.equal(panel.error, null)
  assert.equal(sent.url, 'https://example.test/fresh')
  assert.equal(sent.method, 'PUT')
  assert.deepEqual(sent.query_params, { added: 'fresh' })
  assert.deepEqual(sent.headers, { 'X-Token': 'fresh' })
  assert.deepEqual(sent.body, { type: 'json', content: '' })
  assert.equal(sent.auth.token, 'fresh')
  assert.deepEqual(testInput.request.query_params, { added: 'fresh' })
  assert.deepEqual(testInput.response.headers['set-cookie'], ['a=1', 'b=2'])
  assert.deepEqual(testInput.variables, { token: 'fresh' })
  assert.equal(panel.requestState.method, 'POST')
  panel.handleUrlUpdate('https://example.test?old=new+value')
  assert.deepEqual(JSON.parse(JSON.stringify(panel.requestState.params)), [{ key: 'old', value: 'new value', enabled: true }])
  assert.equal(original.params[0].value, 'old')
  const body = panel.applyVariableReplacement({ path: 'C:\\files\\"quoted"' }, '', [], [], {
    type: 'form-data', content: JSON.stringify([{ key: 'file', value: '{{path}}', type: 'file', enabled: true }]),
  }).body
  assert.equal(JSON.parse(body.content)[0].value, 'C:\\files\\"quoted"')
  unmount()
  await nextTick()
  tabs.persist()
})


test('each send reads fresh environment values without retaining pre-request or test variables', async (t) => {
  for (const preRequestScript of ['', 'pre']) {
    await t.test(preRequestScript ? 'pre-request and test scripts' : 'test script only', async () => {
      const pinia = freshPinia()
      const envs = useEnvironmentsStore()
      envs.environments = [{ id: '__global__', name: 'Global', variables: { token: 'old', removable: 'present' } }]
      const sent = []
      const testInputs = []
      globalThis.__TAURI_INTERNALS__ = { invoke: async (command, args) => {
        if (command === 'get_collections') return []
        if (command === 'execute_script') {
          const input = JSON.parse(args.input)
          if (input.script_type === 'pre_request') {
            assert.equal(input.variables.local, undefined)
            assert.equal(input.variables.fromTest, undefined)
            assert.equal(input.variables.removable, 'present')
            const variables = { ...input.variables }
            if (sent.length === 0) {
              variables.local = 'temporary'
              delete variables.removable
            }
            return JSON.stringify({ variables, modified_request: input.request })
          }
          testInputs.push(input)
          return JSON.stringify({ variables: { ...input.variables, fromTest: 'temporary' } })
        }
        if (command === 'send_request') {
          sent.push(args.input)
          return { success: true, status: 200, headers: {}, body: '', time_ms: 1 }
        }
        throw new Error(command)
      } }
      const { panel, unmount } = mountPanel({
        ...structuredClone(DEFAULT_REQUEST_STATE), url: 'https://example.test/{{token}}',
        preRequestScript, testScript: 'test',
      }, pinia)
      try {
        await panel.sendRequest()
        assert.equal(panel.error, null)
        envs.environments[0].variables.token = 'new'
        await panel.sendRequest()
        assert.equal(panel.error, null)
        assert.deepEqual(sent.map(input => input.url), ['https://example.test/old', 'https://example.test/new'])
        assert.equal(testInputs[0].variables.local, preRequestScript ? 'temporary' : undefined)
        assert.equal(testInputs[0].variables.removable, preRequestScript ? undefined : 'present')
        assert.deepEqual(testInputs[1].variables, { token: 'new', removable: 'present' })
        assert.deepEqual(envs.environments[0].variables, { token: 'new', removable: 'present' })
      } finally { unmount() }
    })
  }
})

test('invalid draft and script form bodies stop sending with a readable error', async () => {
  const invalid = ['not JSON', '{}', 'null', '[null]', '[1]', '[{}]',
    '[{"key":1,"value":"x"}]', '[{"key":"x","value":1}]',
    '[{"key":"x","value":"y","enabled":"false"}]',
    '[{"key":"x","value":"y","type":"invalid"}]']
  for (const type of ['form-data', 'x-www-form-urlencoded']) {
    for (const preRequestScript of ['', 'pre']) {
      for (const content of invalid) {
        const pinia = freshPinia()
        let sends = 0
        globalThis.__TAURI_INTERNALS__ = { invoke: async (command, args) => {
          if (command === 'get_collections') return []
          if (command === 'execute_script') {
            const input = JSON.parse(args.input)
            return JSON.stringify({ variables: {}, modified_request: { ...input.request, body: content } })
          }
          if (command === 'send_request') sends++
          throw new Error(command)
        } }
        const { panel, unmount } = mountPanel({
          ...structuredClone(DEFAULT_REQUEST_STATE), url: 'https://example.test', preRequestScript,
          body: { type, content: preRequestScript ? '[]' : content },
        }, pinia)
        try {
          await panel.sendRequest()
          const message = i18n.global.t('request.invalidFormBody', { type })
          assert.equal(panel.error, preRequestScript ? i18n.global.t('request.preRequestError', { message }) : message)
          assert.equal(sends, 0)
          assert.equal(panel.loading, false)
        } finally { unmount() }
      }
    }
  }
})

test('structured script form bodies preserve types, escape variable values and support clearing', async () => {
  for (const type of ['form-data', 'x-www-form-urlencoded']) {
    const pinia = freshPinia()
    const value = 'C:\\files\\"quoted" & value'
    let content = JSON.stringify([
      { key: '{{field}}', value: '{{value}}', type: 'text' },
      { key: 'off', value: 'ignored', enabled: false },
    ])
    let sent
    globalThis.__TAURI_INTERNALS__ = { invoke: async (command, args) => {
      if (command === 'get_collections') return []
      if (command === 'execute_script') return JSON.stringify({
        variables: { field: 'name', value }, modified_request: { body: content },
      })
      if (command === 'send_request') {
        sent = args.input
        return { success: true, status: 200, headers: {}, body: '', time_ms: 1 }
      }
      throw new Error(command)
    } }
    const { panel, unmount } = mountPanel({
      ...structuredClone(DEFAULT_REQUEST_STATE), url: 'https://example.test',
      preRequestScript: 'pre', body: { type, content: '[]' },
    }, pinia)
    try {
      await panel.sendRequest()
      assert.equal(panel.error, null)
      assert.equal(sent.body.type, type)
      assert.deepEqual(JSON.parse(sent.body.content), [{ key: 'name', value, type: 'text', enabled: true }])
      content = ''
      await panel.sendRequest()
      assert.equal(panel.error, null)
      assert.deepEqual(sent.body, { type, content: '[]' })
    } finally { unmount() }
  }
})

test('import dialog persists every parsed request field through the backend command', async () => {
  const pinia = freshPinia()
  const { default: ImportExport } = await import('../src/components/common/ImportExport.vue')
  const creates = []
  globalThis.__TAURI_INTERNALS__ = { invoke: async (command, { input }) => {
    creates.push({ command, input })
    return { ...input, id: command === 'create_collection' ? 'collection-id' : 'request-id', created_at: '', updated_at: '', position: 0 }
  } }
  const app = renderer.createApp({ ...ImportExport, render: () => null }, { mode: 'import' })
  app.use(pinia).use(i18n)
  const panel = app.mount({}).$.setupState
  panel.importText = JSON.stringify({
    info: { name: 'Import' }, item: [{ name: 'Full', request: {
      method: 'POST', url: 'https://example.test?x=1', header: [{ key: 'X-Test', value: 'yes' }],
      body: { mode: 'raw', raw: '{"x":1}' }, auth: { type: 'bearer', bearer: [{ key: 'token', value: 'secret' }] },
    }, event: [{ listen: 'prerequest', script: { exec: ['pre'] } }, { listen: 'test', script: { exec: ['test'] } }] }],
  })
  await panel.handleImport()
  assert.equal(panel.importError, '')
  const saved = creates.find(c => c.command === 'create_request').input
  assert.equal(saved.collection_id, 'collection-id')
  assert.equal(JSON.parse(saved.params)[0].value, '1')
  assert.equal(JSON.parse(saved.headers)[0].key, 'X-Test')
  assert.equal(JSON.parse(saved.body).content, '{"x":1}')
  assert.equal(JSON.parse(saved.auth).bearer.token, 'secret')
  assert.equal(saved.pre_request_script, 'pre')
  assert.equal(saved.test_script, 'test')
  app.unmount()
})

test('cookie display keeps both Set-Cookie lines despite a comma in Expires', async () => {
  const { default: ResponsePanel } = await import('../src/components/response/ResponsePanel.vue')
  const app = renderer.createApp({ ...ResponsePanel, render: () => null }, {
    loading: false, response: { status: 200, status_text: 'OK', body: '', time_ms: 1, size_bytes: 0,
      headers: { 'set-cookie': ['a=1; Expires=Wed, 21 Oct 2030 07:28:00 GMT', 'b=2; Path=/'] } },
  })
  app.use(i18n)
  const panel = app.mount({}).$.setupState
  assert.deepEqual(panel.cookies, { a: '1', b: '2' })
  app.unmount()
})
