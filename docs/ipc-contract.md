# ClipVault V2 — IPC 契约表 + 架构补全

> Phase 2 产出物，由架构师 [Claude] 输出。

## 1. IPC 类型映射表（Rust ↔ TypeScript）

### 1.1 枚举类型

| Rust 类型 | serde 序列化值 | TypeScript 类型 | 当前状态 |
|-----------|---------------|-----------------|---------|
| `ContentType::Text` | `"text"` | `"text"` | **不对齐** — TS 当前用 `'Text'` |
| `ContentType::Image` | `"image"` | `"image"` | **不对齐** — TS 当前用 `'Image'` |
| `ContentType::FilePath` | `"file_path"` | `"file_path"` | **不对齐** — TS 当前用 `'FilePath'` |

**修复方案**: TS 端改为 `'text' | 'image' | 'file_path'`（匹配 Rust snake_case 输出）

### 1.2 数据结构

| 字段 | Rust 类型 | serde 行为 | TS 当前类型 | TS 正确类型 |
|------|----------|-----------|------------|------------|
| Clip.id | `String` | 直接 | `string` | `string` ✅ |
| Clip.content_type | `ContentType` | snake_case | `'Text'\|'Image'\|'FilePath'` | `'text'\|'image'\|'file_path'` ❌ |
| Clip.content | `Vec<u8>` | base64 编码 | `number[]` | `string` ❌ |
| Clip.preview | `String` | 直接 | `string` | `string` ✅ |
| Clip.content_hash | `String` | 直接 | `string` | `string` ✅ |
| Clip.is_favorite | `bool` | 直接 | `boolean` | `boolean` ✅ |
| Clip.created_at | `i64` | 直接 | `number` | `number` ✅ |
| ClipSummary | (无 content/content_hash) | — | 包含 content/content_hash | 去掉这两个字段 ❌ |

## 2. IPC Command 契约表

### 2.1 已实现 Command

| # | Command | Rust 签名 | TS 函数 | 对齐状态 |
|---|---------|-----------|---------|---------|
| C1 | `list_clips` | `(limit: u32, offset: u32, content_type: Option<ContentType>) → Vec<ClipSummary>` | `listClips({limit, offset})` | ❌ TS 缺 content_type 参数 |
| C2 | `search_clips` | `(query: String, content_type: Option<ContentType>, limit: u32) → Vec<ClipSummary>` | `searchClips(query, limit)` | ❌ TS 缺 content_type 参数 |
| C3 | `get_clip` | `(id: String) → Clip` | `getClip(id)` | ❌ TS Clip 类型不对齐 |
| C4 | `delete_clip` | `(id: String) → ()` | `deleteClip(id)` | ✅ |
| C5 | `delete_clips` | `(ids: Vec<String>) → ()` | — | ❌ TS 未定义 |
| C6 | `toggle_favorite` | `(id: String) → ClipSummary` | `toggleFavorite(id): Promise<Clip>` | ❌ 返回类型不对 |
| C7 | `purge_clips` | `(keep_days: Option<u32>, keep_count: Option<u32>) → u64` | `purgeClips(keepCount)` | ⚠️ 缺 keep_days |
| C8 | `get_config` | `() → AppConfig` | — | ❌ TS 未定义 |
| C9 | `update_config` | `(config: AppConfig) → AppConfig` | — | ❌ TS 未定义 |
| C10 | `create_clip` | `(content_type: ContentType, content: Vec<u8>) → Clip` | — | ❌ TS 未定义 |

### 2.2 待实现 Command

| # | Command | 签名 | 说明 |
|---|---------|------|------|
| C11 | `start_monitor` | `() → ()` | 启动剪贴板监控 |
| C12 | `stop_monitor` | `() → ()` | 停止监控 |
| C13 | `monitor_status` | `() → MonitorStatus` | 查询监控状态 |
| C14 | `paste_clip` | `(id: String) → ()` | 粘贴到活跃应用（当前 TODO） |

### 2.3 Channel 事件

| 事件名 | 方向 | 载荷类型 | 说明 |
|--------|------|---------|------|
| `clip-created` | Rust → TS | `ClipSummary` | 新记录推送 |

## 3. 新增依赖清单

### Cargo.toml 新增

```toml
arboard = "3"          # 跨平台剪贴板读写（替代 cocoa/objc FFI）
enigo = "0.2"          # 键盘模拟（Cmd+V 粘贴）
```

### package.json 新增

```
lucide-react           # 图标库（替代 emoji）
@/components/ui/*      # shadcn/ui 组件（手动添加）
```

## 4. 缺失模块实现清单

### 4a. Backend 缺失

| 文件 | 当前状态 | 需实现内容 |
|------|---------|-----------|
| `services/monitor_service.rs` | 仅注释 | arboard 轮询 + hash 检测 + Channel 推送 |
| `commands/clipboard.rs` paste_clip | TODO 错误 | arboard 写入 + enigo 模拟 Cmd+V |
| `commands/mod.rs` | 缺 start/stop/status | 新增 monitor 模块或直接在 mod.rs 加命令 |
| `lib.rs` | 缺 Channel 注册 | AppHandle 传递 + Channel 创建 |
| `lib.rs` | 缺热键注册 | global_shortcut 注册 Cmd+Shift+V |
| `lib.rs` | 缺托盘设置 | TrayIconBuilder + 右键菜单 |
| `lib.rs` | 缺窗口关闭拦截 | on_window_event → 隐藏不退出 |

### 4b. Frontend 缺失

| 文件 | 当前状态 | 需修改内容 |
|------|---------|-----------|
| `api/clips.ts` | 类型不对齐 | 修复 ContentType/content/toggleFavorite 返回 |
| `api/clips.ts` | 缺函数 | 新增 pasteClip/startMonitor/stopMonitor |
| `views/ClipVaultView.tsx` | emoji 按钮 | 改用 lucide 图标 |
| `views/ClipVaultView.tsx` | 无 Enter 粘贴 | 选中项 Enter 触发 paste + 关闭窗口 |
| `views/ClipVaultView.tsx` | 无类型筛选 | 添加 content_type 筛选 UI |
| `views/ClipVaultView.tsx` | 无搜索高亮 | 搜索结果中高亮匹配关键词 |
| `components/` | 空目录 | 添加 shadcn/ui 组件 |
| `hooks/useClips.ts` | 基本实现 | 添加 pasteClip 集成 |

### 4c. Integration 缺失

| 功能 | 当前状态 | 实现方案 |
|------|---------|---------|
| 全局热键 | lib.rs 有插件无注册 | global_shortcut().on_shortcut() 注册 |
| 系统托盘 | tauri.conf.json 有配置无代码 | TrayIconBuilder + Menu |
| 窗口管理 | 配置有无边框/置顶 | on_window_event(CloseRequested) → hide |
| 粘贴模拟 | paste_clip TODO | arboard 写入 + enigo 模拟 Cmd+V |

## 5. Capabilities 权限更新

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "core:window:allow-close",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "shell:allow-open",
    "store:allow-get",
    "store:allow-set"
  ]
}
```

无新增权限 — 剪贴板操作通过 Rust FFI (arboard) 完成，不需要 `clipboard-read` 插件权限。

## 6. 数据流图

```
[系统剪贴板变更]
    ↓ arboard 轮询 (250ms)
[monitor_service: hash 对比]
    ↓ 新内容
[clip_service: 去重 + 写入 SQLite]
    ↓ Channel::send()
[前端: listen('clip-created')]
    ↓ 更新列表

[用户: Cmd+Shift+V]
    ↓ global_shortcut handler
[Rust: show/focus 窗口]

[用户: Enter 选中]
    ↓ IPC invoke('paste_clip', {id})
[Rust: arboard 写入 + enigo Cmd+V]
    ↓ hide 窗口
```
