# ClipVault 技术设计

## 1. 架构图

```
┌──────────────────────────────────────────────────────────────┐
│                      Frontend (WebView)                       │
│  React 19 + TypeScript + Tailwind + shadcn/ui + Zustand      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────────┐  │
│  │ SearchBar│ │ ClipList │ │ Preview  │ │ Settings       │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────┬─────────┘  │
└───────┼────────────┼────────────┼───────────────┼────────────┘
        │ Tauri IPC (invoke)      │               │
        ▼            ▼            ▼               ▼
┌──────────────────────────────────────────────────────────────┐
│                     Tauri Command Layer                       │
│  commands::clipboard    commands::search    commands::prefs  │
└──────────────────────────────┬───────────────────────────────┘
                               │
┌──────────────────────────────▼───────────────────────────────┐
│                      Service Layer (pure Rust)                │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────────┐ │
│  │ ClipService   │  │ SearchService │  │ MonitorService   │ │
│  │ CRUD + dedup  │  │ FTS5 wrapper  │  │ clipboard watch  │ │
│  └───────┬───────┘  └───────┬───────┘  └────────┬─────────┘ │
└──────────┼──────────────────┼────────────────────┼───────────┘
           │                  │                    │
┌──────────▼──────────────────▼────────────────────▼───────────┐
│                   Repository Layer (spawn_blocking)           │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ ClipRepository  (rusqlite + FTS5)                    │    │
│  │ Arc<Mutex<Connection>>   ──►  spawn_blocking         │    │
│  └──────────────────────────┬───────────────────────────┘    │
└─────────────────────────────┼────────────────────────────────┘
                              │
┌─────────────────────────────▼────────────────────────────────┐
│                        SQLite (local file)                    │
│  clips / clips_fts / clip_tags                                │
└──────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼────────────────────────────────┐
│                        OS Layer                               │
│  macOS: NSPasteboard (cocoa/objc)                             │
│  Windows: Win32 Clipboard (windows-rs)                        │
│  Global Hotkey (tauri-plugin-global-shortcut)                 │
│  System Tray (tauri-plugin-shell)                             │
└──────────────────────────────────────────────────────────────┘
```

## 2. 数据模型

### SQLite Schema

详见 [schema.sql](./schema.sql)。

核心表：

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `clips` | 剪贴板记录 | id (TEXT UUID), content_type, content (BLOB), preview, content_hash, is_favorite, created_at |
| `clips_fts` | FTS5 全文搜索虚拟表 | rowid → clips.id, content |
| `clip_tags` | 标签关联 (P2) | clip_id, tag |

### 数据模型 (Rust)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub content_type: ContentType,
    pub content: Vec<u8>,
    pub preview: String,
    pub content_hash: String,
    pub is_favorite: bool,
    pub created_at: i64, // Unix timestamp ms
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    FilePath,
}
```

## 3. IPC Command 列表

### 3.1 剪贴板记录

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `list_clips` | `limit: u32, offset: u32, content_type: Option<ContentType>` | `Vec<ClipSummary>` | 分页列表，不含 content 字段 |
| `get_clip` | `id: String` | `Clip` | 获取完整记录（含图片二进制） |
| `create_clip` | `content_type: ContentType, content: Vec<u8>` | `Clip` | 手动创建记录（含去重） |
| `delete_clip` | `id: String` | `()` | 删除单条 |
| `delete_clips` | `ids: Vec<String>` | `()` | 批量删除 |
| `toggle_favorite` | `id: String` | `Clip` | 切换收藏状态 |
| `purge_clips` | `keep_days: Option<u32>, keep_count: Option<u32>` | `u64` | 自动清理，返回删除条数 |

### 3.2 搜索

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `search_clips` | `query: String, content_type: Option<ContentType>, limit: u32` | `Vec<ClipSummary>` | FTS5 全文搜索 |
| `search_count` | `query: String, content_type: Option<ContentType>` | `u64` | 匹配条数（用于分页） |

### 3.3 剪贴板监控

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `start_monitor` | `()` | `()` | 启动剪贴板监控线程 |
| `stop_monitor` | `()` | `()` | 停止监控 |
| `monitor_status` | `()` | `MonitorStatus` | 当前监控状态 |
| `on_clip_change` | `Channel<ClipSummary>` | `()` | 剪贴板变更事件推送 |

### 3.4 偏好设置

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `get_config` | `()` | `AppConfig` | 读取配置 |
| `update_config` | `config: AppConfig` | `AppConfig` | 更新配置 |
| `export_data` | `()` | `String` (JSON) | 导出所有记录 |
| `import_data` | `json: String` | `u64` | 导入记录，返回去重后新增条数 |

### 3.5 粘贴

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `paste_clip` | `id: String` | `()` | 将记录内容写入系统剪贴板并模拟粘贴 |

## 4. Capabilities 权限声明

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "shell:allow-open",
    "store:allow-get",
    "store:allow-set"
  ]
}
```

权限最小化原则：
- 不申请 `clipboard-read` 插件权限（通过 Rust FFI 直接调用系统 API，更高效）
- 文件读写仅限应用数据目录（由 Rust 端控制路径）
- 无网络权限（纯离线应用）

## 5. 模块职责划分

```
src-tauri/src/
├── commands/           # Tauri IPC 薄适配器
│   ├── mod.rs          # Command 注册 + generate_handler!
│   ├── clipboard.rs    # 剪贴板 CRUD + 粘贴
│   ├── search.rs       # FTS5 搜索
│   ├── monitor.rs      # 监控控制 + Channel 事件
│   └── prefs.rs        # 配置 + 导入导出
├── models/
│   └── mod.rs          # Clip, ContentType, ClipSummary, MonitorStatus, AppConfig
├── services/
│   ├── mod.rs
│   ├── clip_service.rs # 去重逻辑、hash 计算、业务校验
│   ├── search_service.rs # FTS5 查询构造 + 结果排序
│   └── monitor_service.rs # 系统剪贴板轮询 + 变更检测
├── repositories/
│   ├── mod.rs
│   └── clip_repo.rs    # spawn_blocking 包装所有 rusqlite 操作
├── error.rs            # AppError (thiserror + Serialize)
├── state.rs            # AppState + AppConfig
├── lib.rs              # Tauri Builder + plugin 注册
└── main.rs             # 桌面入口
```

### 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 数据库访问 | `Arc<Mutex<Connection>>` + `spawn_blocking` | SQLite Connection 非 Sync，Mutex 保证 Send+Sync，spawn_blocking 不阻塞 tokio runtime |
| 去重 | SHA-256 content_hash + UNIQUE 约束 | 连续复制相同内容不产生重复，数据库层保证 |
| 剪贴板监控 | 250ms 轮询 + content hash 对比 | 跨平台兼容，CPU 占用可控 |
| 全文搜索 | FTS5 + BM25 排序 | SQLite 内建，零额外依赖，10000 条 < 300ms |
| 图片存储 | BLOB 直接存 SQLite | 单文件部署，避免文件系统碎片，< 5MB 单张 |
| 进程间通信 | Tauri IPC invoke + Channel 事件 | invoke 请求/响应，Channel 剪贴板变更推送 |
