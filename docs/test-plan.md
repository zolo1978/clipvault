# ClipVault 测试计划

## 1. Rust 后端单元测试

### 1.1 clip_service 测试（优先级：P0）

| 测试用例 | 覆盖函数 | 验证点 |
|----------|----------|--------|
| `test_create_text_clip` | `create_clip` | 创建文本 clip，验证 id/preview/hash 正确 |
| `test_create_image_clip` | `create_clip` | 创建图片 clip，验证缩略图生成 |
| `test_create_dedup` | `create_clip` | 相同内容不创建重复记录 |
| `test_create_empty_content` | `create_clip` | 空内容返回 Validation 错误 |
| `test_get_clip_found` | `get_clip` | 按 ID 查询返回完整 clip |
| `test_get_clip_not_found` | `get_clip` | 不存在的 ID 返回 None |
| `test_delete_clip` | `delete_clip` | 删除后查询返回 NotFound |
| `test_delete_clips_batch` | `delete_clips` | 批量删除 ≤500 条 |
| `test_delete_clips_limit` | `delete_clips` | 超过 500 条返回 Validation 错误 |
| `test_toggle_favorite` | `toggle_favorite` | 切换后 is_favorite 翻转 |
| `test_purge_by_days` | `purge_clips` | 清理 N 天前的记录 |
| `test_purge_by_count` | `purge_clips` | 保留最近 N 条 |
| `test_compute_hash` | `compute_hash` | 相同输入相同输出，不同输入不同输出 |
| `test_make_preview_text` | `make_preview` | 文本截断到 200 字符 |
| `test_make_preview_image` | `make_preview` | 图片返回 data URI |
| `test_generate_thumbnail` | `generate_thumbnail` | 缩略图宽度 ≤200px，保持比例 |
| `test_to_rgba_rgb` | `to_rgba` | RGB 3 通道转 RGBA 4 通道 |
| `test_to_rgba_grayscale` | `to_rgba` | 灰度转 RGBA |
| `test_now_ms` | `now_ms` | 返回毫秒时间戳，合理范围 |

### 1.2 clip_repo 测试（优先级：P0）

| 测试用例 | 覆盖函数 | 验证点 |
|----------|----------|--------|
| `test_insert_and_list` | `insert_clip` + `list_clips` | 插入后能列出 |
| `test_list_pagination` | `list_clips` | limit/offset 分页正确 |
| `test_list_filter_type` | `list_clips` | contentType 过滤正确 |
| `test_search_basic` | `search_clips` | FTS5 全文搜索 |
| `test_search_chinese` | `search_clips` | 中文搜索 |
| `test_search_empty_query` | `search_clips` | 空查询返回空结果 |
| `test_search_special_chars` | `search_clips` | 特殊字符不崩溃 |

### 1.3 models 测试（优先级：P1）

| 测试用例 | 覆盖函数 | 验证点 |
|----------|----------|--------|
| `test_content_type_roundtrip` | `ContentType` | 文本/图片/路径 序列化反序列化一致 |
| `test_clip_summary_serialize` | `ClipSummary` | JSON 序列化字段正确 |
| `test_base64_roundtrip` | base64 模块 | 二进制 → base64 → 二进制 一致 |

---

## 2. Rust 集成测试

### 2.1 命令层测试（优先级：P1）

| 测试用例 | 覆盖命令 | 验证点 |
|----------|----------|--------|
| `test_paste_clip_text` | `paste_clip` | 文本粘贴流程正确 |
| `test_paste_clip_not_text` | `paste_clip` | 非文本返回 Validation 错误 |
| `test_paste_clip_not_found` | `paste_clip` | 不存在的 ID 返回 NotFound |
| `test_paste_clip_reentry_guard` | `paste_clip` | 并发粘贴被拒绝 |
| `test_list_clips_limit_max` | `list_clips` | limit=1000 不崩溃 |
| `test_get_config` | `get_config` | 返回默认配置 |
| `test_update_config` | `update_config` | 更新后读取一致 |
| `test_update_config_invalid` | `update_config` | max_clips=0 被拒绝 |

---

## 3. 前端测试

### 3.1 工具函数测试（优先级：P0）

| 测试用例 | 覆盖函数 | 验证点 |
|----------|----------|--------|
| `test_data_uri_to_blob` | `dataUriToBlob` | 有效 data URI → blob URL |
| `test_data_uri_invalid` | `dataUriToBlob` | 无效输入返回原字符串 |
| `test_format_time_now` | `formatTime` | <60s 显示 "刚刚" |
| `test_format_time_minutes` | `formatTime` | 5min → "5分钟前" |
| `test_format_time_hours` | `formatTime` | 3h → "3小时前" |
| `test_format_time_days` | `formatTime` | 2d → "2天前" |
| `test_highlight_text_match` | `HighlightText` | 匹配文本高亮 |
| `test_highlight_text_empty` | `HighlightText` | 空查询不高亮 |
| `test_safe_invoke_success` | `safeInvoke` | 成功调用返回数据 |
| `test_safe_invoke_error` | `safeInvoke` | IPC 错误转为 Error 抛出 |

### 3.2 Hook 测试（优先级：P1）

| 测试用例 | 覆盖逻辑 | 验证点 |
|----------|----------|--------|
| `test_use_clips_initial_load` | `useClips` | 初始化加载 clips |
| `test_use_clips_search` | `useClips` | search 调用 searchClips API |
| `test_use_clips_delete` | `useClips` | 删除后列表更新 |
| `test_use_clips_toggle_fav` | `useClips` | 切换收藏后状态更新 |
| `test_use_clips_paste_error` | `useClips` | 粘贴失败设置 error |
| `test_use_clips_filter_change` | `useClips` | 切换 filterType 触发重新加载 |
| `test_use_clips_event_listener` | `useClips` | clip-created 事件触发刷新 |

### 3.3 组件测试（优先级：P2）

| 测试用例 | 覆盖组件 | 验证点 |
|----------|----------|--------|
| `test_clip_item_text` | ClipItem | 文本预览显示 |
| `test_clip_item_image` | ClipItem | 图片缩略图渲染 |
| `test_clip_item_double_click` | ClipItem | 双击触发对应操作 |
| `test_search_input` | SearchBar | 输入触发搜索 |
| `test_filter_tabs` | FilterTabs | 切换过滤类型 |
| `test_quit_confirm_modal` | QuitModal | 确认弹窗显示/隐藏 |

---

## 4. 手动冒烟测试

### 4.1 核心流程（每次发版必测）

| # | 步骤 | 预期结果 |
|---|------|----------|
| 1 | 启动应用 | 窗口出现，圆角，无白边 |
| 2 | 复制一段文字 | ClipVault 列表出现新条目 |
| 3 | 点击该条目 | 窗口隐藏，文字粘贴到之前的 app |
| 4 | Cmd+Shift+V | 窗口重新出现 |
| 5 | 搜索文字 | 列表过滤显示匹配结果 |
| 6 | 切换"图片" tab | 显示图片类型条目，无闪烁 |
| 7 | 点击收藏按钮 | 星标高亮 |
| 8 | 点击删除按钮 | 条目消失 |
| 9 | 点击截图按钮 | 截图模式，截取后出现在列表 |
| 10 | 点击红色关闭按钮 | 弹出确认退出对话框 |
| 11 | 点"退出" | 应用完全退出 |
| 12 | 从 Dock 重新启动 | 正常启动 |

### 4.2 边界场景

| # | 步骤 | 预期结果 |
|---|------|----------|
| 1 | 复制空字符串 | 不出现在列表 |
| 2 | 复制超长文本（10000+ 字） | 预览截断，不崩溃 |
| 3 | 复制图片（截图） | 缩略图正常显示 |
| 4 | 快速连续复制 10 次 | 全部捕获，不崩溃 |
| 5 | 粘贴时目标 app 已关闭 | 不崩溃，窗口恢复 |
| 6 | 数据库文件不存在时启动 | 自动创建 |
| 7 | 系统主题切换（深色/浅色） | 跟随切换 |
| 8 | 托盘菜单 "显示面板" | 窗口出现 |
| 9 | 托盘菜单 "退出" | 应用退出 |

### 4.3 性能验证

| # | 场景 | 指标 |
|---|------|------|
| 1 | 1000 条记录时列表滚动 | 无卡顿 |
| 2 | 搜索响应时间 | <100ms |
| 3 | 内存占用（闲置 10 分钟） | <50MB |
| 4 | 冷启动时间 | <2s |

---

## 5. CI 自动化

### 已配置（.github/workflows/ci.yml）

- [x] `cargo fmt --check`
- [x] `cargo clippy -- -D warnings`
- [x] `cargo build --release`
- [x] `npx tsc --noEmit`

### 待添加

- [ ] `cargo test` — Rust 单元/集成测试
- [ ] `npx vitest run` — 前端测试
- [ ] 覆盖率报告上传

---

## 执行计划

| 阶段 | 内容 | 预估 |
|------|------|------|
| Phase 1 | Rust 单元测试（clip_service + clip_repo） | 3h |
| Phase 2 | 前端工具函数 + hook 测试 | 2h |
| Phase 3 | 集成测试 + CI 集成 | 2h |
| Phase 4 | 手动冒烟测试 | 1h |
| Phase 5 | 组件测试 + 性能测试 | 2h |
