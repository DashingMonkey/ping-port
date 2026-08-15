export interface Tab {
  id: string
  requestId: string
  method: string
  title: string
  position: number
}

export interface Draft {
  id: string
  tabId: string
  sourceType: 'new' | 'request'
  sourceId?: string
  state: RequestState
  originalState?: RequestState
  changesCount: number
}

export interface RequestState {
  method: HttpMethod
  url: string
  params: KeyValuePair[]
  headers: KeyValuePair[]
  body: RequestBody
  auth: AuthConfig
  preRequestScript: string
  testScript: string
}

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'OPTIONS' | 'HEAD'

export type RequestSubTab = 'params' | 'headers' | 'body' | 'auth' | 'scripts'

export interface KeyValuePair {
  key: string
  value: string
  enabled: boolean
  type?: 'text' | 'file'
}

export interface RequestBody {
  type: 'none' | 'json' | 'form-data' | 'x-www-form-urlencoded'
  content: string
}

export interface AuthConfig {
  type: 'none' | 'basic' | 'bearer' | 'api-key'
  basic?: { username: string; password: string }
  bearer?: { token: string }
  apiKey?: { key: string; value: string; in: 'header' | 'query' }
}

export const DEFAULT_REQUEST_STATE: RequestState = {
  method: 'GET',
  url: '',
  params: [{ key: '', value: '', enabled: true }],
  headers: [{ key: 'Content-Type', value: 'application/json', enabled: true }],
  body: { type: 'none', content: '' },
  auth: { type: 'none' },
  preRequestScript: '',
  testScript: '',
}
