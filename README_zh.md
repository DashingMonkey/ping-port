# PingPort

一款轻量、快速的 API 调试工具，基于 Tauri + Vue 3 + Rust 构建。

## 功能特性

* **HTTP 请求**：支持 GET、POST、PUT、DELETE、PATCH、OPTIONS

* **请求体**：无、JSON、Form-data、URL-encoded

* **认证方式**：无认证、Basic Auth、Bearer Token、API Key

* **集合管理**：将请求组织到文件夹中

* **工作区**：多个独立的数据库（`.db` 文件）

* **环境变量**：全局和环境级变量，支持 `{{变量名}}` 替换

* **前后置脚本**：JavaScript 脚本，提供 `pp` API

* **MCP 集成**：让 AI（Claude、Cursor、Trae 等）通过 Model Context Protocol 直接管理集合和请求

* **类 VS Code 界面**：浅色/深色主题、可调整大小的面板、标签页、无边框窗口（大小/位置持久化）

## 技术栈

| 层级       | 技术                 |
| -------- | ------------------ |
| 桌面框架     | Tauri 2            |
| 前端       | Vue 3 + TypeScript |
| 样式       | Tailwind CSS       |
| 状态管理     | Pinia              |
| HTTP 客户端 | reqwest (Rust)     |
| 数据库      | SQLite (rusqlite)  |
| 脚本引擎     | rquickjs (QuickJS) |

## 目录结构

```
ping-port/
├── src/                          # Vue 3 前端（components、stores、composables）
└── src-tauri/                    # Rust 后端
    └── src/
        ├── commands/             # Tauri 命令处理（薄 IPC 层）
        ├── services/             # 核心业务层（commands 与 MCP 共用）
        ├── db/                   # SQLite 表结构、迁移、连接
        ├── mcp/                   # MCP 服务：HTTP JSON-RPC + 工具 + stdio 桥接
        ├── http/                 # HTTP 客户端（reqwest，支持认证）
        └── scripting/            # JavaScript 沙箱（rquickjs）与 pp.* API
```

## 数据存储

### SQLite 数据库（按工作区）

每个工作区是位于可执行文件目录下的独立 `.db` 文件。

| 表名             | 说明                                                   |
| -------------- | ---------------------------------------------------- |
| `collections`  | 文件夹层级结构（id、parent\_id、name、sort\_order）              |
| `requests`     | HTTP 请求（method、url、params、headers、body、auth、scripts） |
| `environments` | 环境名称和变量（JSON 格式）                                     |

### 浏览器 localStorage（按工作区）

| 键名                            | 说明                  |
| ----------------------------- | ------------------- |
| `pingport-{workspace}-tabs`   | 打开的标签页状态            |
| `pingport-{workspace}-drafts` | 未保存的请求更改            |
| `pingport-settings`           | 全局设置（主题、当前环境、UI 状态） |

### 变量解析优先级（三层）

1. **标签页级** - 在请求编辑器中设置的变量
2. **集合级** - 从父集合继承的变量
3. **全局/环境级** - 从选中环境获取的变量

## 依赖列表

### 前端（package.json）

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

### 后端（Cargo.toml）

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

## 构建

```bash
# 安装前端依赖
npm install

# 开发模式运行
npm run tauri dev

# 生产环境构建
npm run tauri build
```

## MCP 集成（AI 接入）

PingPort 内置 MCP 服务，AI 客户端（Claude Code、Cursor、Claude Desktop、Trae、VS Code Copilot 等）可以直接查看、新增、修改集合和请求，界面实时同步刷新。

### 工作原理

```
AI 客户端 --stdio--> PingPort.exe mcp-bridge --HTTP--> PingPort 应用（内置 MCP 服务）--> SQLite
```

* PingPort 运行时会在 `127.0.0.1` 随机端口启动本地 MCP HTTP 服务（随机端口 + Bearer token，连接信息写入 `%APPDATA%\com.pingport.app\mcp.json`）。

* `PingPort.exe mcp-bridge` 是轻量 stdio 桥接进程：从 AI 客户端读取 MCP 消息并转发给运行中的应用。

* 应用未运行时，桥接进程会自动拉起应用。安装版和绿色版均可使用——配置里指向 exe 的实际路径即可。

* HTTP 端点使用简化的 JSON-RPC（`POST /mcp`，无 SSE 流、无 `Mcp-Session-Id`、不支持批量请求）。它是内置 stdio 桥接的内部传输，并非标准 MCP Streamable HTTP 服务——AI 客户端应通过 `mcp-bridge`（stdio）连接，而不是直接走 HTTP。

### 配置方法

Claude Code：

```bash
claude mcp add pingport -- "C:/Path/To/PingPort.exe" mcp-bridge
```

Cursor / Claude Desktop / Trae 等使用 JSON 配置的客户端（按平台调整可执行文件路径）：

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

### 可用工具

| 工具                  | 说明                                    |
| ------------------- | ------------------------------------- |
| `list_collections`  | 列出当前工作区的集合和请求（含 ID）                   |
| `list_requests`     | 列出已保存的请求，params/headers/body/auth 已解析 |
| `create_collection` | 新建集合（文件夹），可通过 `parent_id` 嵌套          |
| `create_request`    | 在已有集合中新建请求                            |
| `update_request`    | 按 ID 部分更新请求                           |
| `delete_request`    | 按 ID 删除请求                             |
| `delete_collection` | 按 ID 删除集合（请求级联删除）                     |

### 注意事项

* 工具操作的是 PingPort 应用**当前激活的工作区**，在界面切换工作区后 AI 的操作对象也随之切换。

* 典型流程：先调用 `list_collections` 获取 ID，再调用 `create_request` / `update_request`。

* 请求字段格式与界面一致：`params`/`headers` 为 `[{key, value, enabled}]` 数组，`body` 为 `{type, content}`，`auth` 为 `{type: none|basic|bearer|api-key, ...}`。`update_request` 中对 `params`/`headers`/`body`/`auth` 传 `null` 可清空该字段。

* 连接信息文件位置可通过环境变量 `PINGPORT_MCP_CONFIG` 覆盖（绿色版自定义目录时有用）。

## 脚本 API

前后置脚本运行在沙箱化的 JavaScript 环境中，提供以下 API：

```javascript
// 变量
pp.variables.get("varname")           // 获取变量
pp.variables.set("varname", "value")  // 设置变量

// 请求（测试脚本中只读）
pp.request.method                     // "GET"、"POST" 等
pp.request.url                        // 原始 URL
pp.request.headers                    // 请求头对象
pp.request.body                       // 请求体

// 响应（仅测试脚本可用）
pp.response.status                    // HTTP 状态码
pp.response.headers                   // 响应头对象
pp.response.body                      // 响应体（字符串）
pp.response.time                      // 响应时间（毫秒）

// 测试
pp.test(name, fn)                     // 定义测试用例
```

## 窗口与界面

- 默认尺寸：1280x800，最小：900x600
- 自定义无边框窗口，带标题栏控制按钮
- 窗口状态持久化（大小、位置）
- 类 VS Code 活动栏和侧边栏布局
