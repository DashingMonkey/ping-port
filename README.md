# PingPort

A lightweight, fast API debugging tool built with Tauri + Vue 3 + Rust.

## Features

* **HTTP Requests**: Support for GET, POST, PUT, DELETE, PATCH, OPTIONS

* **Request Body**: None, JSON, Form-data, URL-encoded

* **Authentication**: None, Basic Auth, Bearer Token, API Key

* **Collections**: Organize requests into folders

* **Workspaces**: Multiple isolated databases (`.db` files)

* **Environment Variables**: Global and per-environment variables with `{{variable}}` substitution

* **Pre-request & Test Scripts**: JavaScript scripting with `pp` API

* **MCP Integration**: Let AI agents (Claude, Cursor, Trae, etc.) manage collections and requests directly via Model Context Protocol

* **VSCode-inspired UI**: Light/dark themes, resizable panels, tabbed interface, frameless window with size/position persistence

## Tech Stack

| Layer             | Technology         |
| ----------------- | ------------------ |
| Desktop Framework | Tauri 2            |
| Frontend          | Vue 3 + TypeScript |
| Styling           | Tailwind CSS       |
| State Management  | Pinia              |
| HTTP Client       | reqwest (Rust)     |
| Database          | SQLite (rusqlite)  |
| Scripting Engine  | rquickjs (QuickJS) |

## Architecture

```
ping-port/
├── src/                          # Vue 3 frontend (components, stores, composables)
└── src-tauri/                    # Rust backend
    └── src/
        ├── commands/             # Tauri command handlers (thin IPC layer)
        ├── services/             # Core business logic (shared by commands & MCP)
        ├── db/                   # SQLite schema, migrations, connection
        ├── mcp/                  # MCP server: HTTP JSON-RPC + tools + stdio bridge
        ├── http/                 # HTTP client (reqwest) with auth support
        └── scripting/            # JavaScript sandbox (rquickjs) & pp.* API
```

## Data Storage

### SQLite Database (Per Workspace)

Each workspace is a separate `.db` file located in the executable directory.

| Table          | Description                                                       |
| -------------- | ----------------------------------------------------------------- |
| `collections`  | Folder hierarchy (id, parent\_id, name, sort\_order)              |
| `requests`     | HTTP requests (method, url, params, headers, body, auth, scripts) |
| `environments` | Environment name and variables (JSON)                             |

### Browser localStorage (Per Workspace)

| Key                           | Description                                           |
| ----------------------------- | ----------------------------------------------------- |
| `pingport-{workspace}-tabs`   | Open tabs state                                       |
| `pingport-{workspace}-drafts` | Unsaved request changes                               |
| `pingport-settings`           | Global settings (theme, active environment, UI state) |

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

## MCP Integration

PingPort ships with a built-in MCP server, so AI agents (Claude Code, Cursor, Claude Desktop, Trae, VS Code Copilot, etc.) can list, create, and update your collections and requests directly — and the UI refreshes in real time.

### How it works

```
AI client --stdio--> PingPort.exe mcp-bridge --HTTP--> PingPort app (local MCP server) --> SQLite
```

* When PingPort runs, it starts a local MCP HTTP server on `127.0.0.1` (random port + bearer token, stored in `%APPDATA%\com.pingport.app\mcp.json`).

* `PingPort.exe mcp-bridge` is a lightweight stdio bridge: it reads MCP messages from the AI client and forwards them to the running app.

* If PingPort is not running, the bridge starts it automatically. Both installed and portable ("green") builds work — just point the config at the executable path.

* The HTTP endpoint speaks simplified JSON-RPC over `POST /mcp` (no SSE streams, no `Mcp-Session-Id`, no request batching). It is an internal transport for the bundled stdio bridge, not a standard MCP Streamable HTTP server — AI clients should connect via `mcp-bridge` (stdio), not directly over HTTP.

### Configuration

Claude Code:

```bash
claude mcp add pingport -- "C:/Path/To/PingPort.exe" mcp-bridge
```

Cursor / Claude Desktop / Trae / other JSON-based clients (adjust the executable path per platform):

```json
{
  "mcpServers": {
    "pingport": {
      "command": "C://Path//To//PingPort.exe",
      "args": ["mcp-bridge"]
    }
  }
}
```

### Available tools

| Tool                | Description                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `list_collections`  | List collections and requests (with IDs) in the current workspace |
| `list_requests`     | List saved requests with parsed params/headers/body/auth          |
| `create_collection` | Create a collection (folder), optionally nested via `parent_id`   |
| `create_request`    | Create a request in an existing collection                        |
| `update_request`    | Partially update a request by ID                                  |
| `delete_request`    | Delete a request by ID                                            |
| `delete_collection` | Delete a collection by ID (requests cascade)                      |

### Notes

* Tools operate on the **currently active workspace** in the PingPort app. Switching workspaces in the UI also switches what the AI sees.

* Typical flow: `list_collections` first to get IDs, then `create_request` / `update_request`.

* Request field formats follow the UI: `params`/`headers` are `[{key, value, enabled}]` arrays, `body` is `{type, content}`, `auth` is `{type: none|basic|bearer|api-key, ...}`. In `update_request`, passing `null` for `params`/`headers`/`body`/`auth` clears the field.

* The connection file location can be overridden with the `PINGPORT_MCP_CONFIG` environment variable (useful for portable setups).

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
