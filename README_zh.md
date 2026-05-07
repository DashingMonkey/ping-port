# PingPort

一款轻量、快速的 API 调试工具，基于 Tauri + Vue 3 + Rust 构建。

## 功能特性

- **HTTP 请求**：支持 GET、POST、PUT、DELETE、PATCH、OPTIONS
- **请求体**：无、JSON、Form-data、URL-encoded
- **认证方式**：无认证、Basic Auth、Bearer Token、API Key
- **集合管理**：将请求组织到文件夹中
- **工作区**：多个独立的数据库（`.db` 文件）
- **环境变量**：全局和环境级变量，支持 `{{变量名}}` 替换
- **前后置脚本**：JavaScript 脚本，提供 `pp` API
- **类 VS Code 界面**：浅色/深色主题、可调整大小的面板、标签页

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript |
| 样式 | Tailwind CSS |
| 状态管理 | Pinia |
| HTTP 客户端 | reqwest (Rust) |
| 数据库 | SQLite (rusqlite) |
| 脚本引擎 | rquickjs (QuickJS) |

## 目录结构

```
ping-port/
├── src/                          # Vue 前端
│   ├── components/               # UI 组件
│   │   ├── layout/              # 布局组件（AppLayout、Header、Sidebar、ActivityBar）
│   │   ├── sidebar/             # 侧边栏组件（CollectionTree、ContextMenu）
│   │   ├── request/            # 请求面板组件（RequestPanel、UrlBar、BodyTab、HeadersTab、AuthTab、ScriptsTab）
│   │   ├── response/            # 响应面板组件（ResponsePanel、ResponseBody、ResponseHeaders、TestResults）
│   │   └── common/             # 通用组件（Toast、ConfirmDialog、EnvironmentManager）
│   ├── stores/                  # Pinia 状态管理
│   │   ├── tabs.ts             # 标签页管理与草稿跟踪
│   │   ├── requests.ts         # 请求 CRUD
│   │   ├── collections.ts       # 集合树管理
│   │   ├── environments.ts     # 环境变量（三层解析）
│   │   ├── settings.ts          # 主题、UI 状态
│   │   └── workspace.ts         # 工作区切换
│   ├── composables/             # Vue 组合式函数
│   └── lib/                     # 工具函数（变量替换、导入/导出）
│
├── src-tauri/                   # Rust 后端
│   ├── src/
│   │   ├── commands/            # Tauri 命令处理
│   │   │   ├── requests.rs      # 请求 CRUD
│   │   │   ├── collections.rs   # 集合 CRUD
│   │   │   ├── environments.rs  # 环境管理
│   │   │   ├── http.rs          # HTTP 请求执行
│   │   │   ├── script.rs        # 前后置脚本执行
│   │   │   └── workspace.rs     # 工作区管理
│   │   ├── db/                  # 数据库层
│   │   │   ├── mod.rs           # SQLite 连接（WAL 模式）
│   │   │   ├── schema.rs        # 表结构定义
│   │   │   └── migrations.rs     # 数据库迁移
│   │   ├── http/
│   │   │   └── client.rs        # HTTP 客户端（支持认证）
│   │   └── scripting/           # JavaScript 引擎
│   │       ├── engine.rs        # rquickjs 运行时
│   │       ├── pm_api.rs        # Postman 风格 pp.* API
│   │       └── isolation.rs      # 脚本沙箱隔离
│   └── Cargo.toml
│
├── public/                       # 静态资源
├── package.json                  # Node 依赖
├── vite.config.ts               # Vite 打包配置
├── tailwind.config.js           # Tailwind 主题配置
└── tauri.conf.json              # Tauri 应用配置
```

## 数据存储

### SQLite 数据库（按工作区）

每个工作区是位于可执行文件目录下的独立 `.db` 文件。

| 表名 | 说明 |
|------|------|
| `collections` | 文件夹层级结构（id、parent_id、name、sort_order） |
| `requests` | HTTP 请求（method、url、params、headers、body、auth、scripts） |
| `environments` | 环境名称和变量（JSON 格式） |

### 浏览器 localStorage（按工作区）

| 键名 | 说明 |
|------|------|
| `pingport-{workspace}-tabs` | 打开的标签页状态 |
| `pingport-{workspace}-drafts` | 未保存的请求更改 |
| `pingport-settings` | 全局设置（主题、当前环境、UI 状态） |

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
