# ClipVault V2 — AC 验收清单

> Phase 1 产出物，由 PM [Claude] 从 PRD 提取。每条 AC 有明确的验证方法和期望结果。

## P0 验收项（MVP 阻塞）

### AC-1: 剪贴板文本自动记录
- **来源**: US-1 AC1
- **验证方法**: 复制一段文本 → 检查 ClipVault 面板
- **期望结果**: 100ms 内面板可见新记录
- **当前状态**: monitor_service.rs 仅有注释占位

### AC-2: hash 去重
- **来源**: US-1 AC2
- **验证方法**: 连续复制相同内容 5 次 → 检查记录数
- **期望结果**: 仅 1 条记录
- **当前状态**: clip_service.rs 有 hash 计算，但监控未实现无法触发

### AC-3: 全局热键唤起
- **来源**: US-2 AC1
- **验证方法**: 在 VS Code 中按 Cmd+Shift+V
- **期望结果**: 150ms 内面板弹出
- **当前状态**: lib.rs 注册了 global-shortcut 插件，但未注册具体热键和处理函数

### AC-4: 搜索响应
- **来源**: US-2 AC2
- **验证方法**: 输入 "docker" → 检查响应时间（1000 条记录内）
- **期望结果**: < 100ms
- **当前状态**: FTS5 search 基本实现，需验证性能

### AC-5: 选择粘贴
- **来源**: US-2 AC3
- **验证方法**: 选中记录按 Enter → 检查 VS Code 光标位置
- **期望结果**: 内容粘贴到光标位置，面板关闭
- **当前状态**: paste_clip 返回 TODO 错误

### AC-6: 托盘图标
- **来源**: US-3 AC1
- **验证方法**: 启动应用 → 检查系统托盘
- **期望结果**: 托盘图标可见，无主窗口弹出
- **当前状态**: Cargo.toml 有 tray-icon feature，但 lib.rs 未配置 TrayIconBuilder

### AC-7: 托盘右键菜单
- **来源**: US-3 AC2
- **验证方法**: 右键托盘图标
- **期望结果**: 菜单含「显示面板 / 暂停监控 / 设置 / 退出」
- **当前状态**: 未实现

### AC-8: 剪贴板变更事件推送
- **来源**: US-1 + tech-design 3.3
- **验证方法**: 复制文本 → 前端监听 `clip-created` 事件
- **期望结果**: 前端自动收到新记录推送，无需轮询
- **当前状态**: Channel 事件未实现

### AC-9: 监控暂停/恢复
- **来源**: US-3 AC3 + tech-design 3.3
- **验证方法**: 托盘点击「暂停监控」→ 复制文本 → 检查是否记录
- **期望结果**: 暂停后不记录，图标变灰色
- **当前状态**: start_monitor/stop_monitor 命令未注册

### AC-10: 关闭窗口不退出
- **来源**: US-3 AC4
- **验证方法**: 关闭主窗口 → 检查托盘图标和进程
- **期望结果**: 应用继续运行，仅窗口隐藏
- **当前状态**: 未实现窗口关闭事件拦截

## P1 验收项（V1.0 必需）

### AC-11: 暗色模式
- **验证方法**: 切换系统主题 → 检查 UI
- **期望结果**: 200ms 内重绘
- **当前状态**: App.tsx 有 initTheme 调用，需验证完整性

### AC-12: 搜索高亮
- **验证方法**: 搜索关键词 → 检查结果列表
- **期望结果**: 匹配关键词高亮显示
- **当前状态**: 未实现

### AC-13: 收藏功能
- **验证方法**: 点击收藏图标 → 检查列表排序
- **期望结果**: 收藏项固定顶部，响应 < 50ms
- **当前状态**: toggle_favorite 命令已实现

### AC-14: 单条删除
- **验证方法**: 删除一条记录 → 检查列表
- **期望结果**: 列表实时更新无闪烁
- **当前状态**: delete_clip 命令已实现

### AC-15: 搜索类型筛选
- **来源**: PRD Section 3 搜索 AC3
- **验证方法**: 选择「仅图片」筛选
- **期望结果**: 仅显示图片类型记录
- **当前状态**: 后端支持 content_type 参数，前端 UI 未实现

## IPC 契约验证项

| # | Command | Rust 签名 | TS 类型 | 对齐状态 |
|---|---------|-----------|---------|---------|
| IC-1 | list_clips | (u32, u32, Option<ContentType>) → Vec<ClipSummary> | 需定义 | 待验证 |
| IC-2 | search_clips | (String, Option<ContentType>, u32) → Vec<ClipSummary> | 需定义 | 待验证 |
| IC-3 | get_clip | (String) → Clip | 需定义 | 待验证 |
| IC-4 | paste_clip | (String) → () | 需定义 | 待实现 |
| IC-5 | toggle_favorite | (String) → ClipSummary | 需定义 | 待验证 |
| IC-6 | start_monitor | () → () | 需定义 | 未实现 |
| IC-7 | stop_monitor | () → () | 需定义 | 未实现 |
| IC-8 | on_clip_change | Channel<ClipSummary> | 需定义 | 未实现 |

## 反模式检测项

| # | 检查项 | grep 规则 |
|---|--------|----------|
| AP-1 | unwrap 使用 | `grep -rn '\.unwrap()' src-tauri/src/` |
| AP-2 | TODO 占位 | `grep -rn 'TODO' src-tauri/src/` |
| AP-3 | clone in loop | `grep -rn '\.clone()' src-tauri/src/` |
| AP-4 | panic 使用 | `grep -rn 'panic!' src-tauri/src/` |
| AP-5 | expect 无消息 | `grep -rn '\.expect("")' src-tauri/src/` |
