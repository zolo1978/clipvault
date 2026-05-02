好的，这是一份针对 ClipVault 桌面应用的 UI 设计规格文档，遵循您提供的技术栈和要求。

---

# ClipVault UI 设计规格

## 技术栈概览
- **框架:** React 19 + TypeScript
- **样式:** Tailwind CSS 4
- **UI 组件库:** shadcn/ui
- **状态管理:** Zustand
- **图标库:** lucide-react
- **窗口尺寸:** 400x600px，无边框 (borderless)，始终置顶 (alwaysOnTop)
- **间距规范:** 4px 的倍数

## 1. 组件清单

以下是 ClipVault 应用的主要 UI 组件及其层级关系、用途、关键 Props，并指明了所使用的 shadcn/ui 基础组件和 lucide-react 图标。

```
App (根组件)
└── ClipVaultView (主要视图布局)
    ├── Header (应用顶部区域)
    │   ├── Title (应用标题)
    │   │   └── Text (shadcn/ui)
    │   ├── SearchBar (搜索输入框)
    │   │   └── Input (shadcn/ui)
    │   │       └── Icon: Search (lucide-react) (作为输入框左侧前缀)
    │   ├── FilterButton (内容类型筛选按钮，如：文本、图片、文件路径)
    │   │   └── DropdownMenu (shadcn/ui)
    │   │       └── Button (shadcn/ui, 带当前筛选类型图标)
    │   │           └── Icon: FileText / Image / Folder / Files / ListFilter (lucide-react, 根据筛选类型变化)
    │   ├── ThemeToggle (亮暗模式切换)
    │   │   └── Button (shadcn/ui)
    │   │       └── Icon: Sun / Moon / Laptop (lucide-react, 根据当前主题变化)
    │   └── SettingsButton (设置按钮)
    │       └── Button (shadcn/ui)
    │           └── Icon: Settings (lucide-react)
    ├── ClipList (剪贴板历史记录列表，可滚动)
    │   └── ScrollArea (shadcn/ui)
    │       └── ClipListItem (单个剪贴板历史记录项)
    │           ├── FavoriteButton (收藏/取消收藏按钮)
    │           │   └── Button (shadcn/ui, icon only)
    │           │       └── Icon: Star (收藏) / StarOff (未收藏) (lucide-react)
    │           ├── ClipPreview (剪贴板内容预览)
    │           │   └── Image (对于图片) / Text (对于文本) / Icon: File / Folder (对于文件路径)
    │           ├── ContentTypeTag (内容类型标签)
    │           │   └── Badge (shadcn/ui)
    │           │       └── Icon: FileText / Image / Folder (lucide-react, 根据类型)
    │           ├── Timestamp (创建时间)
    │           │   └── Text (shadcn/ui, muted-foreground 样式)
    │           └── DeleteButton (删除单条记录按钮)
    │               └── Button (shadcn/ui, icon only, destructive 样式)
    │                   └── Icon: Trash2 (lucide-react)
    ├── ContextMenu (剪贴板列表项右键菜单)
    │   └── ContextMenu (shadcn/ui)
    │       ├── ContextMenuItem (粘贴)
    │       │   └── Icon: ClipboardPaste (lucide-react)
    │       ├── ContextMenuItem (收藏/取消收藏)
    │       │   └── Icon: Star (lucide-react)
    │       ├── ContextMenuItem (删除)
    │       │   └── Icon: Trash2 (lucide-react)
    │       └── ContextMenuItem (复制内容)
    │           └── Icon: Copy (lucide-react)
    └── SettingsDialog (设置对话框)
        └── Dialog (shadcn/ui)
            ├── DialogHeader (对话框标题)
            │   └── DialogTitle (shadcn/ui)
            │   └── DialogDescription (shadcn/ui)
            ├── ConfigForm (表单区域)
            │   ├── Label (shadcn/ui) + Switch (shadcn/ui) (剪贴板监控开关)
            │   ├── Label (shadcn/ui) + Select (shadcn/ui) (自动清理策略)
            │   ├── Label (shadcn/ui) + Input (shadcn/ui) (全局快捷键显示，不可编辑)
            │   └── Button (shadcn/ui, primary 样式) (保存配置)
            └── DialogFooter (对话框底部)
                └── Button (shadcn/ui) (关闭)

**常用 shadcn/ui 组件列表:**
- `Button`: 用于所有可点击的交互元素。
- `Input`: 搜索框、配置项输入。
- `Dialog`: 设置、确认删除等模态框。
- `DropdownMenu`: 筛选、更多操作菜单。
- `ScrollArea`: 剪贴板列表滚动区域。
- `Separator`: 分隔线。
- `Switch`: 布尔值配置项。
- `Label`: 表单字段标签。
- `Badge`: 内容类型展示。
- `Tooltip`: 提示信息。

**常用 lucide-react 图标列表:**
- `Search`: 搜索输入框
- `Sun`, `Moon`, `Laptop`: 主题切换
- `Settings`: 设置按钮
- `Star`, `StarOff`: 收藏/取消收藏
- `Trash2`: 删除按钮
- `FileText`: 文本类型
- `Image`: 图片类型
- `Folder`, `File`: 文件路径类型
- `ListFilter`: 筛选按钮默认图标
- `ClipboardPaste`: 粘贴操作
- `Copy`: 复制操作
- `RefreshCcw`: 刷新（如果需要）

## 2. Design Token

以下是 ClipVault 的设计令牌，包含颜色、间距、字体、圆角和阴影，并对应 Tailwind CSS 类名。所有间距均基于 4px 的倍数。

### 颜色系统 (基于 HSL，用于 light/dark 主题切换)

在 `tailwind.config.js` 中配置，并通过 CSS 变量定义。

```css
:root {
  --background: 0 0% 100%; /* White */
  --foreground: 222.2 47.4% 11.2%; /* Almost black */
  --card: 0 0% 100%;
  --card-foreground: 222.2 47.4% 11.2%;
  --popover: 0 0% 100%;
  --popover-foreground: 222.2 47.4% 11.2%;
  --primary: 221.2 83.2% 53.3%; /* Blue */
  --primary-foreground: 210 20% 98%; /* White */
  --secondary: 210 40% 96.1%; /* Light gray */
  --secondary-foreground: 222.2 47.4% 11.2%;
  --muted: 210 40% 96.1%;
  --muted-foreground: 215.4 16.3% 46.9%; /* Gray */
  --accent: 210 40% 96.1%;
  --accent-foreground: 222.2 47.4% 11.2%;
  --destructive: 0 84.2% 60.2%; /* Red */
  --destructive-foreground: 210 20% 98%;
  --border: 214.3 31.8% 91.4%; /* Light border gray */
  --input: 214.3 31.8% 91.4%;
  --ring: 221.2 83.2% 53.3%;
  --radius: 0.5rem; /* 8px */
}

.dark {
  --background: 222.2 47.4% 11.2%;
  --foreground: 210 20% 98%;
  --card: 222.2 47.4% 11.2%;
  --card-foreground: 210 20% 98%;
  --popover: 222.2 47.4% 11.2%;
  --popover-foreground: 210 20% 98%;
  --primary: 217.2 91.2% 59.8%;
  --primary-foreground: 222.2 47.4% 11.2%;
  --secondary: 217.2 32.6% 17.5%;
  --secondary-foreground: 210 20% 98%;
  --muted: 217.2 32.6% 17.5%;
  --muted-foreground: 215 20.2% 65.1%;
  --accent: 217.2 32.6% 17.5%;
  --accent-foreground: 210 20% 98%;
  --destructive: 0 62.8% 30.6%;
  --destructive-foreground: 210 20% 98%;
  --border: 217.2 32.6% 17.5%;
  --input: 217.2 32.6% 17.5%;
  --ring: 217.2 91.2% 59.8%;
}
```

- **语义色:**
    - `primary`: 主要交互元素 (按钮背景, 焦点环)
        - Tailwind Class: `bg-primary`, `text-primary`, `border-primary`, `ring-primary`
    - `secondary`: 次要交互元素 (次要按钮, 辅助背景)
        - Tailwind Class: `bg-secondary`, `text-secondary`, `border-secondary`
    - `muted`: 柔和背景或文字 (次要文本, 分隔符)
        - Tailwind Class: `bg-muted`, `text-muted-foreground`
    - `accent`: 强调色 (hover 状态)
        - Tailwind Class: `bg-accent`, `text-accent-foreground`
    - `destructive`: 危险操作 (删除按钮)
        - Tailwind Class: `bg-destructive`, `text-destructive-foreground`

- **表面色:**
    - `background`: 整体背景色
        - Tailwind Class: `bg-background`
    - `card`: 卡片、面板背景
        - Tailwind Class: `bg-card`
    - `popover`: 弹窗、菜单背景
        - Tailwind Class: `bg-popover`

- **边框色:**
    - `border`: 边框、分隔线
        - Tailwind Class: `border`
    - `input`: 输入框边框
        - Tailwind Class: `border-input`

- **文字色:**
    - `foreground`: 主要文本颜色
        - Tailwind Class: `text-foreground`
    - `muted-foreground`: 次要文本颜色、提示文本
        - Tailwind Class: `text-muted-foreground`

### 间距系统 (4px 基数)

通过 Tailwind 的默认间距刻度，其都是 4px 的倍数。
- `p-1` (4px), `p-2` (8px), `p-3` (12px), `p-4` (16px), `p-5` (20px), `p-6` (24px), `p-8` (32px), `p-10` (40px), `p-12` (48px)
- `m-` 系列 (外边距) 同上
- `gap-` 系列 (Flexbox/Grid 间距) 同上
- `w-` / `h-` 系列 (宽度/高度) 同上

### 字体系统

- **字体家族:** 采用系统默认无衬线字体，或指定 `Inter` 字体。
    - Tailwind Class: `font-sans`
- **字号:**
    - `text-xs`: 12px
    - `text-sm`: 14px
    - `text-base`: 16px (默认)
    - `text-lg`: 18px
    - `text-xl`: 20px
- **字重:**
    - `font-normal`: 400
    - `font-medium`: 500
    - `font-semibold`: 600
    - `font-bold`: 700

### 圆角

- `rounded-none`: 0px
- `rounded-sm`: 2px
- `rounded`: 4px
- `rounded-md`: 6px
- `rounded-lg`: 8px (主要按钮、卡片)
- `rounded-xl`: 12px
- `rounded-full`: 9999px (圆形)

### 阴影

- `shadow-sm`: box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);
- `shadow`: box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);
- `shadow-md`: box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
- `shadow-lg`: box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
- `shadow-xl`: box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);

## 3. 状态设计

ClipVault 的状态管理将通过 Zustand 实现，拆分为三个主要 Store：`clipStore`、`uiStore` 和 `configStore`。

### 3.1 Zustand Stores

#### `clipStore.ts` (剪贴板数据管理)
- **State:**
    - `clips: ClipSummary[]`: 当前加载的剪贴板摘要列表。
    - `selectedClipId: string | null`: 当前被选中的 Clip 的 ID (用于键盘导航或详情显示)。
    - `hasMoreClips: boolean`: 是否还有更多剪贴板数据可供加载 (用于无限滚动)。
    - `isLoading: boolean`: 剪贴板数据是否正在加载中。
- **Actions:**
    - `fetchClips(limit: number, offset: number, content_type?: ContentType)`: 从后端 IPC 获取剪贴板摘要列表并更新 `clips`。
    - `searchClips(query: string, content_type?: ContentType, limit?: number)`: 根据搜索查询从后端 IPC 搜索剪贴板。
    - `addClip(clip: ClipSummary)`: 当接收到 Tauri 的 `clipboard-update` 事件时，将新的剪贴板项添加到 `clips` 列表顶部。
    - `removeClip(id: string)`: 从 `clips` 列表中移除指定 ID 的剪贴板。
    - `updateClip(id: string, updates: Partial<ClipSummary>)`: 更新指定 ID 的剪贴板项的属性 (如收藏状态)。
    - `toggleFavorite(id: string)`: 调用后端 IPC `toggle_favorite` 并更新本地状态。
    - `setSelectedClip(id: string)`: 设置当前选中的剪贴板 ID。
    - `clearSelectedClip()`: 清除选中的剪贴板 ID。
    - `selectNextClip()`: 键盘向下选择下一个剪贴板项。
    - `selectPreviousClip()`: 键盘向上选择上一个剪贴板项。
    - `pasteSelectedClip()`: 调用后端 IPC `paste_clip` 粘贴当前选中的剪贴板。

#### `uiStore.ts` (用户界面状态管理)
- **State:**
    - `theme: 'light' | 'dark' | 'system'`: 当前应用主题模式。
    - `isSettingsOpen: boolean`: 设置对话框是否打开。
    - `searchQuery: string`: 搜索框的当前输入值。
    - `filterContentType: 'all' | ContentType`: 当前应用的剪贴板类型过滤器。
    - `isWindowVisible: boolean`: 应用窗口是否可见。
- **Actions:**
    - `setTheme(theme: 'light' | 'dark' | 'system')`: 设置应用主题。
    - `toggleSettings(isOpen: boolean)`: 控制设置对话框的显示/隐藏。
    - `setSearchQuery(query: string)`: 更新搜索框内容。
    - `setFilterContentType(type: 'all' | ContentType)`: 设置内容类型过滤器。
    - `toggleWindowVisibility()`: 切换窗口的可见状态。

#### `configStore.ts` (用户配置管理)
- **State:**
    - `autoCleanPolicy: 'never' | '1week' | '1month' | '1day'`: 自动清理策略。
    - `monitorClipboard: boolean`: 是否开启剪贴板监控。
    - `globalHotkey: string`: 全局快捷键字符串。
- **Actions:**
    - `loadConfig()`: 从后端 IPC `get_config` 加载用户配置。
    - `updateConfig(newConfig: Partial<Config>)`: 调用后端 IPC `update_config` 更新用户配置并同步本地状态。

### 3.2 与 Tauri IPC 的数据流图

```mermaid
graph TD
    subgraph Frontend (React Components + Zustand Stores)
        UI[UI Components] -->|交互 (点击, 输入, 快捷键)| uiStore[uiStore]
        UI -->|交互 (列表选择, 收藏, 删除)| clipStore[clipStore]
        UI -->|交互 (设置修改)| configStore[configStore]
    end

    subgraph Backend (Tauri Rust Core)
        IPC_Commands[Tauri IPC Commands]
        MonitorService[Monitor Service (监听系统剪贴板)]
    end

    uiStore -->|调用 IPC commands| IPC_Commands
    clipStore -->|调用 IPC commands| IPC_Commands
    configStore -->|调用 IPC commands| IPC_Commands

    IPC_Commands -->|返回数据| clipStore
    IPC_Commands -->|返回数据| configStore

    MonitorService -->|Tauri Event: clipboard-update| clipStore
```

**数据流说明:**

1.  **用户交互 -> Zustand Stores:**
    *   用户在 UI 上进行操作 (如搜索、点击收藏、修改设置)，这些操作会触发对应的 Zustand Store 的 actions。
2.  **Zustand Stores -> Tauri IPC Commands:**
    *   `clipStore` 通过调用 Tauri 的 `invoke` 函数，触发后端 Rust 的 `list_clips`, `search_clips`, `toggle_favorite`, `delete_clip`, `delete_clips`, `paste_clip` 等命令。
    *   `uiStore` 在需要控制监控服务时，调用 `start_monitor`, `stop_monitor`, `monitor_status`。
    *   `configStore` 调用 `get_config` 和 `update_config` 来读取和保存用户配置。
3.  **Tauri IPC Commands -> Zustand Stores:**
    *   后端 Rust 命令执行完成后，会将结果 (如 `Vec<ClipSummary>`, `Clip`, `Config`) 返回给调用它的前端 Zustand Store，更新相应的状态。
4.  **Monitor Service (Backend) -> Zustand Stores (通过 Tauri Event):**
    *   Tauri 后端的 `monitor_service` 会持续监听系统剪贴板。
    *   当检测到新的剪贴板内容时，它会触发一个 Tauri 事件 (例如 `clipboard-update`)。
    *   前端 `clipStore` 会监听这个事件，接收到新剪贴板数据后，调用 `addClip` action 将其添加到 `clips` 列表中。

## 4. 交互规范

### 4.1 键盘操作

-   **全局快捷键 (Cmd+Shift+V / Ctrl+Shift+V):**
    -   按下快捷键时，如果 ClipVault 窗口隐藏，则显示窗口并自动聚焦到 `SearchBar`。
    -   如果 ClipVault 窗口已显示，则再次按下快捷键会隐藏窗口。
-   **在 ClipList 中:**
    -   `ArrowUp` (↑): 向上移动选中项，循环到列表末尾。
    -   `ArrowDown` (↓): 向下移动选中项，循环到列表开头。
    -   `Enter`: 粘贴当前选中项的内容到活跃应用，并隐藏 ClipVault 窗口。
-   **在任意状态下:**
    -   `Cmd+F` (macOS) / `Ctrl+F` (Windows/Linux): 聚焦 `SearchBar`。
    -   `Escape` (Esc):
        1.  如果 `SearchBar` 中有文本，清除搜索文本。
        2.  如果搜索文本为空且有任何对话框（如设置对话框）打开，关闭对话框。
        3.  如果以上条件都不满足，则隐藏 ClipVault 窗口。
    -   `Tab` / `Shift+Tab`: 在可交互元素之间切换焦点 (搜索框、筛选按钮、列表项、设置按钮等)。

### 4.2 鼠标操作

-   **ClipList 项:**
    -   **单击:** 选中剪贴板项。如果该项是图片，可以在列表区域直接显示缩略图，并提供全尺寸预览的弹出选项。
    -   **双击:** 粘贴当前选中项的内容到活跃应用，并隐藏 ClipVault 窗口。
    -   **右键单击:** 弹出 `ContextMenu`，提供 "粘贴", "收藏/取消收藏", "删除", "复制内容" 等选项。
-   **FavoriteButton (收藏按钮):** 单击切换剪贴板项的收藏状态。
-   **DeleteButton (删除按钮):** 单击删除剪贴板项。需要弹出确认对话框。
-   **FilterButton (筛选按钮):** 单击打开内容类型筛选下拉菜单。
-   **ThemeToggle (主题切换):** 单击切换主题 (亮色/暗色/跟随系统)。
-   **SettingsButton (设置按钮):** 单击打开设置对话框。
-   **窗口拖拽:** 由于窗口是无边框的，用户可以通过拖拽窗口的顶部区域或自定义拖拽区域来移动窗口。

### 4.3 搜索 Debounce 策略

-   当用户在 `SearchBar` 中输入时，不应立即触发后端搜索 IPC 命令。
-   应设置一个约 `300ms` 的 debounce 延迟。即，在用户停止输入 300ms 后，才调用 `search_clips` IPC 命令进行搜索。这可以减少不必要的后端请求，提高性能和用户体验。

### 4.4 窗口显隐逻辑

-   **显示:**
    -   按下全局快捷键 (Cmd+Shift+V / Ctrl+Shift+V)。
    -   应用启动时 (如果配置为开机启动并显示)。
-   **隐藏:**
    -   再次按下全局快捷键。
    -   通过 `Enter` 或双击粘贴剪贴板内容后。
    -   按下 `Escape` 键 (根据上述 `Escape` 逻辑)。
    -   点击系统托盘图标 (如果实现)。
-   **alwaysOnTop 行为:** 窗口始终保持在其他应用之上，但可以通过快捷键方便地隐藏和显示。点击 ClipVault 窗口外部不应隐藏窗口，因为它是 `alwaysOnTop`。

### 4.5 焦点管理

-   **窗口显示时:** 当 ClipVault 窗口通过全局快捷键显示时，自动将焦点设置到 `SearchBar`。
-   **ClipList 导航:** 使用 `ArrowUp`/`ArrowDown` 导航时，确保当前选中项始终在可见区域内。
-   **对话框打开时:** 当设置对话框或其他模态对话框打开时，焦点应自动转移到对话框内的第一个可交互元素，以便用户可以直接使用键盘进行操作。
-   **对话框关闭时:** 焦点应返回到对话框打开前的位置，或者返回到主界面的 `SearchBar`。

---
