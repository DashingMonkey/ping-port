# PingPort

A lightweight, fast API debugging tool built with Tauri + Vue 3 + Rust.

## Features

- **HTTP Requests**: Support for GET, POST, PUT, DELETE, PATCH, OPTIONS
- **Request Body**: None, JSON, Form-data, URL-encoded
- **Authentication**: None, Basic Auth, Bearer Token, API Key
- **Collections**: Organize requests into folders
- **Workspaces**: Multiple isolated databases (`.db` files)
- **Environment Variables**: Global and per-environment variables with `{{variable}}` substitution
- **Pre-request & Test Scripts**: JavaScript scripting with `pp` API
- **VSCode-inspired UI**: Light/dark themes, resizable panels, tabbed interface

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Framework | Tauri 2 |
| Frontend | Vue 3 + TypeScript |
| Styling | Tailwind CSS |
| State Management | Pinia |
| HTTP Client | reqwest (Rust) |
| Database | SQLite (rusqlite) |
| Scripting Engine | rquickjs (QuickJS) |

## Architecture

```
ping-port/
├── src/                          # Vue frontend
│   ├── components/               # UI components
│   │   ├── layout/              # AppLayout, Header, Sidebar, ActivityBar
│   │   ├── sidebar/             # CollectionTree, ContextMenu
│   │   ├── request/            # RequestPanel, UrlBar, BodyTab, HeadersTab, AuthTab, ScriptsTab
│   │   ├── response/            # ResponsePanel, ResponseBody, ResponseHeaders, TestResults
│   │   └── common/             # Toast, ConfirmDialog, EnvironmentManager
│   ├── stores/                  # Pinia stores
│   │   ├── tabs.ts             # Tab management & draft tracking
│   │   ├── requests.ts         # Request CRUD
│   │   ├── collections.ts     # Collection tree management
│   │   ├── environments.ts     # Environment variables (3-tier resolution)
│   │   ├── settings.ts         # Theme, UI state
│   │   └── workspace.ts       # Workspace switching
│   ├── composables/             # Vue composables
│   └── lib/                    # Utilities (variables, import/export)
│
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── commands/           # Tauri command handlers
│   │   │   ├── requests.rs     # Request CRUD
│   │   │   ├── collections.rs   # Collection CRUD
│   │   │   ├── environments.rs # Environment management
│   │   │   ├── http.rs         # HTTP request execution
│   │   │   ├── script.rs       # Pre-request/test script execution
│   │   │   └── workspace.rs    # Workspace management
│   │   ├── db/                # Database layer
│   │   │   ├── mod.rs         # SQLite connection (WAL mode)
│   │   │   ├── schema.rs      # Table definitions
│   │   │   └── migrations.rs  # Schema migrations
│   │   ├── http/
│   │   │   └── client.rs      # HTTP client with auth support
│   │   └── scripting/         # JavaScript engine
│   │       ├── engine.rs      # rquickjs runtime
│   │       ├── pm_api.rs      # Postman-like pp.* API
│   │       └── isolation.rs   # Script sandboxing
│   └── Cargo.toml
│
├── public/                      # Static assets
├── package.json                 # Node dependencies
├── vite.config.ts              # Vite bundler config
├── tailwind.config.js          # Tailwind theme config
└── tauri.conf.json             # Tauri app config
```

## Data Storage

### SQLite Database (Per Workspace)

Each workspace is a separate `.db` file located in the executable directory.

| Table | Description |
|-------|-------------|
| `collections` | Folder hierarchy (id, parent_id, name, sort_order) |
| `requests` | HTTP requests (method, url, params, headers, body, auth, scripts) |
| `environments` | Environment name and variables (JSON) |

### Browser localStorage (Per Workspace)

| Key | Description |
|-----|-------------|
| `pingport-{workspace}-tabs` | Open tabs state |
| `pingport-{workspace}-drafts` | Unsaved request changes |
| `pingport-settings` | Global settings (theme, active environment, UI state) |

### Variable Resolution (3-Tier Priority)

1. **Tab-level** - Variables set in the request editor
2. **Collection-level** - Variables inherited from parent collection
3. **Global/Environment** - Variables from selected environment

## Dependencies

### Frontend (package.json)

```
vue@^3.5.32
pinia@^3.0.4
@headlessui/vue
@tauri-apps/api@^2.0.0
@tauri-apps/cli@^2.0.0
@vitejs/plugin-vue@^5.0.0
tailwindcss
typescript
vite
vue-tsc
```

### Backend (Cargo.toml)

```
tauri@2
tauri-plugin-shell
tauri-plugin-window-state
rusqlite (bundled)
reqwest
rquickjs
tokio (full)
uuid
chrono
serde/serde_json
log/env_logger
anyhow
rfd
regex
```

## Build

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Scripting API

Pre-request and test scripts run in a sandboxed JavaScript environment with the following API:

```javascript
// Variables
pp.variables.get("varname")           // Get variable
pp.variables.set("varname", "value")   // Set variable

// Request (read-only in tests)
pp.request.method                      // "GET", "POST", etc.
pp.request.url                         // Original URL
pp.request.headers                     // Request headers object
pp.request.body                        // Request body

// Response (tests only)
pp.response.status                     // HTTP status code
pp.response.headers                    // Response headers object
pp.response.body                       // Response body (string)
pp.response.time                       // Response time (ms)

// Tests
pp.test(name, fn)                      // Define test case
```

## Window & UI

- Default size: 1280x800, minimum: 900x600
- Custom frameless window with title bar controls
- Window state persistence (size, position)
- VSCode-inspired activity bar and sidebar layout
