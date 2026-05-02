# ClipVault V2 执行规划 — 多模型团队重做

> 基于上次复盘教训，使用增强后的 9 Agent + 2 Bridge + 10 Skill 团队重做。

## 一、上次失败分析 vs 本次对策

| 上次问题 | 根因 | 本次对策 | 负责 Agent |
|---------|------|---------|-----------|
| 编译通过但不能用 | PRD 写完没验收 | QA Agent 每阶段验收，FAIL 阻塞 | rust-qa-agent [Claude] |
| UI 用 emoji 当按钮 | 没有设计阶段直接写代码 | UI Designer 先出设计规格再实现 | rust-ui-designer-agent [Gemini] |
| 剪贴板/热键/托盘全是 TODO | 缺系统集成专家 | Integration Agent 负责实现 | rust-integration-agent [Codex] |
| IPC 类型不对齐 | 没有接口契约 | Architect 输出契约表，QA 验证 | rust-architect-agent + rust-qa-agent [Claude] |
| 敏感内容未处理 | 安全是盲区 | Security Skill 指导过滤层 | rust-security-skill |
| 性能指标没测 | 没有量化基线 | Performance Skill 提供基线和 benchmark | rust-performance-skill |

## 二、执行流水线（6 Phase）

```
Phase 1: PM [Claude] — PRD 已有，提取 AC 验收清单
    ↓ 质量门: AC 每条可执行
Phase 2: Architect [Claude] — 接口契约 + 依赖清单 + 数据模型
    ↓ 质量门: IPC 契约表完整，依赖清单无遗漏
Phase 3: UI Designer [Gemini] — 组件/Token/状态/交互设计规格
    ↓ 质量门: 设计规格四节齐全
Phase 4: Backend + Frontend + Integration [Codex] — 代码实现（可并行）
    ↓ 质量门: cargo check + vite build 0 error
Phase 5: QA [Claude] — PRD AC 验收 + IPC 契约验证 + 反模式 + Smoke Test
    ↓ FAIL → 退回 Phase 4 | PASS → 继续
Phase 6: Build [Codex] — 签名 + 打包 + 发布
```

## 三、Phase 详细分解

### Phase 1: PRD AC 提取 [Claude, 15min]

**输入：** 已有 `docs/prd.md`（199 行）
**输出：** 可执行的 AC 验收清单

| AC # | 来源 | 验证方法 | 期望结果 |
|------|------|---------|---------|
| AC-1 | US-1 AC1 | 复制文本 → 面板可见 | ≤ 100ms |
| AC-2 | US-1 AC2 | 连续复制同内容 5 次 | 仅 1 条记录 |
| AC-3 | US-2 AC1 | Cmd+Shift+V 唤起 | ≤ 150ms |
| AC-4 | US-2 AC2 | 搜索 "docker" | ≤ 100ms/1000条 |
| AC-5 | US-2 AC3 | Enter 粘贴到 VS Code | 内容正确 |
| AC-6 | US-3 AC1 | 启动后托盘图标 | 仅托盘无窗口 |
| AC-7 | US-3 AC2 | 右键托盘菜单 | 显示/暂停/退出 |
| AC-8 | 暗色模式 | 切换主题 | 200ms 内重绘 |
| AC-9 | 搜索高亮 | 搜索关键词 | 结果高亮显示 |
| AC-10 | 收藏 | 点击收藏 | 顶部固定 |

### Phase 2: 接口契约 + 架构 [Claude, 30min]

**输出：**
1. IPC 契约表（每条 Command 的 Rust ↔ TS 类型映射）
2. Cargo.toml 依赖清单
3. SQLite schema（已有，需验证）
4. Capabilities 权限声明

**上次遗漏项（本次必须补齐）：**
- `paste_clip` 的完整实现（arboard + enigo）
- `start_monitor` / `stop_monitor` 的完整实现
- `on_clip_change` Channel 事件推送
- Capabilities 最小权限验证

### Phase 3: UI 设计规格 [Gemini, 30min]

**输出：** 设计规格 Markdown（四节）

| 节 | 上次问题 | 本次要求 |
|----|---------|---------|
| 组件清单 | emoji 当按钮 | lucide-react 图标 + shadcn/ui 组件 |
| Design Token | 无规范 | 颜色/间距/字体具体值（4px 基数） |
| 状态设计 | 组件内 useState | Zustand Store 统一管理 |
| 交互规范 | 键盘导航不完整 | ArrowUp/Down/Enter/Escape 全覆盖 |

**重点改进：**
- 搜索栏：debounce 300ms + 搜索图标 + 清除按钮
- 列表项：内容预览 + 类型标签 + 时间戳 + 收藏星 + 删除按钮
- 主题切换：lucide Sun/Moon 图标（非 emoji）
- 空状态：插画 + 引导文案
- 加载状态：Skeleton 骨架屏

### Phase 4: 代码实现 [Codex, 并行]

**子任务分配：**

#### 4a: Backend [Codex] — 60min

| 任务 | 文件 | 上次状态 | 本次要求 |
|------|------|---------|---------|
| Clipboard 监控 | monitor_service.rs | TODO 占位 | arboard 轮询 + hash 去重 |
| 粘贴功能 | commands/clipboard.rs | TODO 占位 | arboard 写入 + enigo 模拟 Cmd+V |
| 全文搜索 | repositories/clip_repo.rs | 基础实现 | FTS5 参数化查询 + 搜索高亮支持 |
| 剪贴板变更事件 | Channel 推送 | 缺失 | `clip-created` 事件推送到前端 |
| 错误处理 | error.rs | unwrap | AppError 统一 + thiserror |

#### 4b: Frontend [Codex] — 60min

| 任务 | 文件 | 上次问题 | 本次要求 |
|------|------|---------|---------|
| 主视图 | ClipVaultView.tsx | emoji 按钮 | lucide 图标 + shadcn 组件 |
| 搜索栏 | 内联 | 无 debounce | 300ms debounce + 搜索图标 |
| 列表项 | 内联 | 纯文字 | 类型标签 + 时间格式化 + 收藏星 |
| 加载/空状态 | 内联 | 文字提示 | Skeleton + 引导插画 |
| Hook | useClips.ts | listen 参数错误 | 修正 + Channel 监听 |
| IPC API | clips.ts | 类型不对齐 | 严格按契约表对齐 |

#### 4c: Integration [Codex] — 45min

| 任务 | 上次状态 | 本次要求 |
|------|---------|---------|
| 全局热键 | 未实现 | tauri-plugin-global-shortcut 注册 |
| 系统托盘 | 未实现 | TrayIconBuilder + 右键菜单 |
| 窗口管理 | 配置存在 | 无边框 + 置顶 + 显示/隐藏切换 |
| 热键响应 | 未实现 | Cmd+Shift+V 切换窗口 |

### Phase 5: QA 验收 [Claude, 30min]

**验收维度：**

| 维度 | 检查项 | 工具 |
|------|--------|------|
| PRD AC | 10 条 AC 逐项验证 | cargo test + 手动测试 |
| IPC 契约 | Rust serde ↔ TS interface 对齐 | grep + QA skill Section 2 |
| 反模式 | unwrap/todo/clone in loop | grep 规则 |
| 安全 | FTS 注入/敏感内容/CSP | security-skill 检查命令 |
| 性能 | 搜索延迟/内存/启动时间 | benchmark |

### Phase 6: 构建发布 [Codex, 15min]

- `cargo build --release` + `npm run build`
- macOS codesign + notarize（如果证书可用）
- .app + .dmg 打包

## 四、对比预估

| 维度 | 上次结果 | 本次预期 | 改进来源 |
|------|---------|---------|---------|
| 编译 | ✅ 通过 | ✅ 通过 | 基线 |
| 可运行 | ❌ 不能用 | ✅ 可用 | Integration Agent |
| UI 质量 | ❌ emoji/无设计 | ✅ lucide+shadcn | UI Designer [Gemini] |
| PRD 对齐 | ❌ 未验收 | ✅ 10 AC 全通过 | QA Agent |
| IPC 类型 | ❌ 不对齐 | ✅ 契约验证通过 | Architect + QA |
| 安全 | ❌ 盲区 | ✅ 审计清单通过 | Security Skill |
| 性能基线 | ❌ 未测量 | ✅ 有量化数据 | Performance Skill |
| 系统集成 | ❌ 全部 TODO | ✅ 热键+托盘+剪贴板 | Integration Agent |

## 五、执行时间线

```
Phase 1: PM [Claude]       ───── 15min
Phase 2: Architect [Claude] ───── 30min
Phase 3: UI [Gemini]        ───── 30min
Phase 4a: Backend [Codex]   ──┐
Phase 4b: Frontend [Codex]  ──┤ 并行 60min
Phase 4c: Integration[Codex]──┘
Phase 5: QA [Claude]        ───── 30min
Phase 6: Build [Codex]      ───── 15min
                                  总计 ≈ 3h
```

## 六、风险

| # | 风险 | 缓解 |
|---|------|------|
| R1 | Codex CLI 超时或输出质量不够 | 重试一次，失败则 Claude 接手 |
| R2 | Gemini 设计规格不符合预期 | Claude 审查修改 |
| R3 | arboard/enigo 编译问题 | 参考 integration-skill 的平台抽象模式 |
| R4 | 429 限流 | 按模型分流，避免单模型瓶颈 |
