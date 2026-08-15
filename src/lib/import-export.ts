import i18n from '../i18n'
import type { KeyValuePair, RequestBody, AuthConfig, RequestState } from '../stores/types'
import type { Collection } from '../stores/collections'
import type { Request } from '../stores/requests'

// Postman v2.1 format types
interface PostmanHeader {
  key: string
  value: string
  disabled?: boolean
}

interface PostmanUrl {
  raw: string
  protocol?: string
  host?: string[]
  path?: string[]
  query?: PostmanHeader[]
}

interface PostmanBody {
  mode?: 'none' | 'raw' | 'formdata' | 'urlencoded'
  raw?: string
  formdata?: PostmanHeader[]
  urlencoded?: PostmanHeader[]
}

interface PostmanRequest {
  method?: string
  header?: PostmanHeader[]
  url?: string | PostmanUrl
  body?: PostmanBody
  auth?: PostmanAuth
}

interface PostmanAuth {
  type?: string
  bearer?: { key: string; value: string }[]
  basic?: { username: string; password: string }[]
  apikey?: { key: string; value: string; in: string }[]
}

interface PostmanItem {
  name: string
  request?: PostmanRequest
  item?: PostmanItem[]
  description?: string
}

export interface PostmanCollection {
  info: {
    name: string
    description?: string
    schema: string
  }
  item: PostmanItem[]
}

/**
 * Convert headers array to object for easier storage
 */
function headersToObject(headers: KeyValuePair[]): Record<string, string> {
  const obj: Record<string, string> = {}
  for (const h of headers) {
    if (h.enabled) {
      obj[h.key] = h.value
    }
  }
  return obj
}

/**
 * Parse URL string into components
 */
function parseUrl(urlStr: string): { host: string[]; path: string[]; query: KeyValuePair[] } {
  try {
    let host: string[] = []
    let path: string[] = []
    let query: KeyValuePair[] = []

    // Handle protocol
    const protocolMatch = urlStr.match(/^(\w+):\/\//)
    if (protocolMatch) {
      urlStr = urlStr.substring(protocolMatch[0].length)
    }

    // Split host and path
    const pathIndex = urlStr.indexOf('/')
    let hostPart: string
    let pathPart: string
    if (pathIndex !== -1) {
      hostPart = urlStr.substring(0, pathIndex)
      pathPart = urlStr.substring(pathIndex + 1)
    } else {
      hostPart = urlStr
      pathPart = ''
    }

    // Split host into parts
    host = hostPart.split('.').filter(Boolean)

    // Handle query parameters
    const queryIndex = pathPart.indexOf('?')
    let pathWithoutQuery: string
    if (queryIndex !== -1) {
      pathWithoutQuery = pathPart.substring(0, queryIndex)
      const queryStr = pathPart.substring(queryIndex + 1)
      pathPart = pathWithoutQuery
      query = queryStr.split('&').map(pair => {
        const [key, ...valueParts] = pair.split('=')
        return {
          key: decodeURIComponent(key),
          value: decodeURIComponent(valueParts.join('=')),
          enabled: true,
        }
      })
    }

    // Split path into parts
    path = pathPart.split('/').filter(Boolean)

    return { host, path, query }
  } catch {
    return { host: [], path: [], query: [] }
  }
}

/**
 * Convert internal RequestState to Postman request format
 */
function requestStateToPostman(state: RequestState): PostmanRequest {
  const headers = headersToObject(state.headers)
  const headerArray: PostmanHeader[] = Object.entries(headers).map(([key, value]) => ({
    key,
    value,
    disabled: false,
  }))

  // Parse URL
  let postmanUrl: PostmanUrl
  if (state.url) {
    const parsed = parseUrl(state.url)
    const protocolMatch = state.url.match(/^(\w+):\/\//)
    const protocol = protocolMatch ? protocolMatch[1] : (state.url.startsWith('https') ? 'https' : 'http')
    postmanUrl = {
      raw: state.url,
      protocol,
      host: parsed.host,
      path: parsed.path,
      query: parsed.query.map(q => ({ key: q.key, value: q.value })),
    }
  } else {
    postmanUrl = { raw: '' }
  }

  // Convert body
  let postmanBody: PostmanBody | undefined
  if (state.body.type === 'json' && state.body.content) {
    postmanBody = { mode: 'raw', raw: state.body.content }
  } else if (state.body.type === 'form-data' || state.body.type === 'x-www-form-urlencoded') {
    const formData = state.body.content ? JSON.parse(state.body.content) : []
    postmanBody = {
      mode: state.body.type === 'form-data' ? 'formdata' : 'urlencoded',
      formdata: formData.map((item: KeyValuePair) => ({
        key: item.key,
        value: item.value,
        disabled: !item.enabled,
      })),
    }
  } else {
    postmanBody = { mode: 'none' }
  }

  // Convert auth
  let postmanAuth: PostmanAuth | undefined
  if (state.auth.type === 'bearer' && state.auth.bearer) {
    postmanAuth = {
      type: 'bearer',
      bearer: [{ key: 'token', value: state.auth.bearer.token }],
    }
  } else if (state.auth.type === 'basic' && state.auth.basic) {
    postmanAuth = {
      type: 'basic',
      basic: [{ username: state.auth.basic.username, password: state.auth.basic.password }],
    }
  } else if (state.auth.type === 'api-key' && state.auth.apiKey) {
    postmanAuth = {
      type: 'apikey',
      apikey: [{ key: state.auth.apiKey.key, value: state.auth.apiKey.value, in: state.auth.apiKey.in }],
    }
  }

  return {
    method: state.method,
    header: headerArray,
    url: postmanUrl,
    body: postmanBody,
    auth: postmanAuth,
  }
}

/**
 * Convert Postman request to internal RequestState
 */
function postmanToRequestState(req: PostmanRequest): Partial<RequestState> {
  const state: Partial<RequestState> = {
    method: (req.method as RequestState['method']) || 'GET',
    url: typeof req.url === 'string' ? req.url : req.url?.raw || '',
    params: [],
    headers: [],
    body: { type: 'none', content: '' },
    auth: { type: 'none' },
    preRequestScript: '',
    testScript: '',
  }

  // Parse headers
  if (req.header) {
    state.headers = req.header.map(h => ({
      key: h.key,
      value: h.value,
      enabled: !h.disabled,
    }))
  }

  // Parse URL query params
  if (typeof req.url !== 'string' && req.url?.query) {
    state.params = req.url.query.map(q => ({
      key: q.key,
      value: q.value,
      enabled: !q.disabled,
    }))
  }

  // Parse body
  if (req.body) {
    if (req.body.mode === 'raw' && req.body.raw) {
      state.body = { type: 'json', content: req.body.raw }
    } else if (req.body.mode === 'formdata' && req.body.formdata) {
      state.body = {
        type: 'form-data',
        content: JSON.stringify(req.body.formdata.map(h => ({
          key: h.key,
          value: h.value,
          enabled: !h.disabled,
        }))),
      }
    } else if (req.body.mode === 'urlencoded' && req.body.urlencoded) {
      state.body = {
        type: 'x-www-form-urlencoded',
        content: JSON.stringify(req.body.urlencoded.map(h => ({
          key: h.key,
          value: h.value,
          enabled: !h.disabled,
        }))),
      }
    } else {
      state.body = { type: 'none', content: '' }
    }
  }

  // Parse auth
  if (req.auth) {
    if (req.auth.type === 'bearer' && req.auth.bearer?.[0]) {
      state.auth = { type: 'bearer', bearer: { token: req.auth.bearer[0].value } }
    } else if (req.auth.type === 'basic' && req.auth.basic?.[0]) {
      state.auth = {
        type: 'basic',
        basic: { username: req.auth.basic[0].username, password: req.auth.basic[0].password },
      }
    } else if (req.auth.type === 'apikey' && req.auth.apikey?.[0]) {
      state.auth = {
        type: 'api-key',
        apiKey: {
          key: req.auth.apikey[0].key,
          value: req.auth.apikey[0].value,
          in: req.auth.apikey[0].in as 'header' | 'query',
        },
      }
    }
  }

  return state
}

/**
 * Build collection tree structure from flat collections and requests
 */
function buildCollectionTree(
  collections: Collection[],
  requests: Request[]
): PostmanItem[] {
  const rootItems: PostmanItem[] = []
  const collectionMap = new Map<string, PostmanItem>()

  // Create collection items
  for (const col of collections) {
    const item: PostmanItem = {
      name: col.name,
      description: col.type !== 'folder' ? col.type : undefined,
      item: [],
    }
    collectionMap.set(col.id, item)
  }

  // Build parent-child relationships
  for (const col of collections) {
    if (col.parentId === null) {
      rootItems.push(collectionMap.get(col.id)!)
    } else {
      const parent = collectionMap.get(col.parentId)
      const child = collectionMap.get(col.id)
      if (parent && child) {
        if (!parent.item) parent.item = []
        parent.item.push(child)
      }
    }
  }

  // Add requests to their collections
  for (const req of requests) {
    const collection = collectionMap.get(req.collectionId)
    if (collection) {
      const requestState: RequestState = {
        method: req.method as RequestState['method'],
        url: req.url,
        params: req.params ? JSON.parse(req.params) : [],
        headers: req.headers ? JSON.parse(req.headers) : [],
        body: req.body ? JSON.parse(req.body) : { type: 'none', content: '' },
        auth: req.auth ? JSON.parse(req.auth) : { type: 'none' },
        preRequestScript: req.preRequestScript || '',
        testScript: req.testScript || '',
      }

      const postmanReq = requestStateToPostman(requestState)
      const requestItem: PostmanItem = {
        name: req.name,
        request: postmanReq,
      }
      if (!collection.item) collection.item = []
      collection.item.push(requestItem)
    }
  }

  return rootItems
}

/**
 * Convert collections and requests to Postman v2.1 collection format
 */
export function toPostmanCollection(
  collections: Collection[],
  requests: Request[]
): PostmanCollection {
  return {
    info: {
      name: i18n.global.t('export.pingPortCollection'),
      description: i18n.global.t('export.exportedFrom'),
      schema: 'https://schema.getpostman.com/json/collection/v2.1.0/collection.json',
    },
    item: buildCollectionTree(collections, requests),
  }
}

interface ParsedCollection {
  id: string
  parentId: string | null
  name: string
  type: string
}

interface ParsedRequest {
  id: string
  collectionId: string
  name: string
  method: string
  url: string
  params: KeyValuePair[]
  headers: KeyValuePair[]
  body: RequestBody
  auth: AuthConfig
  preRequestScript: string
  testScript: string
}

/**
 * Recursively parse Postman items to extract collections and requests
 */
function parsePostmanItems(
  items: PostmanItem[],
  collections: ParsedCollection[],
  requests: ParsedRequest[],
  parentId: string | null,
  collectionId: string
): void {
  for (const item of items) {
    if (item.request) {
      // This is a request
      const requestState = postmanToRequestState(item.request)
      const request: ParsedRequest = {
        id: crypto.randomUUID(),
        collectionId,
        name: item.name,
        method: requestState.method || 'GET',
        url: requestState.url || '',
        params: requestState.params || [],
        headers: requestState.headers || [],
        body: requestState.body || { type: 'none', content: '' },
        auth: requestState.auth || { type: 'none' },
        preRequestScript: requestState.preRequestScript || '',
        testScript: requestState.testScript || '',
      }
      requests.push(request)
    }

    if (item.item && item.item.length > 0) {
      // This is a folder/collection
      const folderCollection: ParsedCollection = {
        id: crypto.randomUUID(),
        parentId,
        name: item.name,
        type: item.description || 'folder',
      }
      collections.push(folderCollection)
      parsePostmanItems(item.item, collections, requests, folderCollection.id, folderCollection.id)
    }
  }
}

/**
 * Parse Postman v2.1 collection format to collections and requests
 */
export function fromPostmanCollection(
  data: PostmanCollection
): { collections: Collection[]; requests: Request[] } {
  const parsedCollections: ParsedCollection[] = []
  const parsedRequests: ParsedRequest[] = []

  const rootCollection: ParsedCollection = {
    id: crypto.randomUUID(),
    parentId: null,
    name: data.info.name,
    type: data.info.description || 'folder',
  }
  parsedCollections.push(rootCollection)

  if (data.item) {
    parsePostmanItems(data.item, parsedCollections, parsedRequests, rootCollection.id, rootCollection.id)
  }

  // Convert to store types
  const collections: Collection[] = parsedCollections.map(c => ({
    id: c.id,
    parentId: c.parentId,
    name: c.name,
    type: c.type,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    position: 0,
  }))

  const requests: Request[] = parsedRequests.map((r, index) => ({
    id: r.id,
    collectionId: r.collectionId,
    name: r.name,
    method: r.method,
    url: r.url,
    params: r.params.length > 0 ? JSON.stringify(r.params) : null,
    headers: r.headers.length > 0 ? JSON.stringify(r.headers) : null,
    body: JSON.stringify(r.body),
    auth: r.auth.type !== 'none' ? JSON.stringify(r.auth) : null,
    preRequestScript: r.preRequestScript || null,
    testScript: r.testScript || null,
    position: index,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }))

  return { collections, requests }
}

/**
 * Export data to formatted JSON string
 */
export function exportToJson(data: unknown): string {
  return JSON.stringify(data, null, 2)
}

/**
 * Import data from JSON string
 */
export function importFromJson(jsonString: string): unknown {
  return JSON.parse(jsonString)
}
