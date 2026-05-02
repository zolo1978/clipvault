# ClipVault V1 复盘报告

> 日期: 2025-05-02 | 三路审查: Rust Code Review + Architecture Review + Security Audit

## 总评

V1 功能完整度 **90%**，但工程质量存在系统性不足：**零测试覆盖**、**多处竞态条件**、**临时文件泄露敏感数据**、**前端 500 行单体组件**。需要一轮 2-3 天的集中修复才能达到生产质量。

---

## 问题统计

| 严重度 | 数量 | 说明 |
|--------|------|------|
| CRITICAL | 5 | 必须立即修复，影响数据安全或正确性 |
| HIGH | 15 | 功能缺陷或架构风险 |
| MEDIUM | 16 | 代码质量或可维护性问题 |
| LOW | 10 | 文档、风格、小改进 |

---

## CRITICAL 问题清单

| # | 问题 | 文件 | 行号 | 审查来源 |
|---|------|------|------|----------|
| C1 | 零测试覆盖 | 全项目 | - | Rust+Arch |
| C2 | 剪贴板粘贴竞态 — monitor 捕获临时粘贴内容 | clipboard.rs | 40-60 | Rust+Security |
| C3 | 临时文件泄露敏感剪贴板图片（明文、永不删除） | clipboard.rs | 91-99 | Rust+Security |
| C4 | 截图临时文件可预测路径 + 竞态窗口 | screenshot.rs | 19-47 | Security |
| C5 | 剪贴板数据未加密存储（SQLite 明文） | 全链路 | - | Security |

### C1: 零测试覆盖

整个项目没有任何测试。`state.rs:48` 的 `new_test()` 方法存在但从未使用。业务逻辑（去重、哈希、FTS 搜索、purge、批量删除）的正确性完全无法验证。

### C2: 粘贴竞态

```
paste_clip: 写入目标文本 → simulate_paste → 恢复原剪贴板
monitor:    同时轮询剪贴板 → 捕获到临时文本 → 写入数据库（重复条目）
```

paste 和 monitor 之间没有协调机制。修复方案：paste 期间设置标志，monitor 检测到时跳过采集。

### C3: 临时文件泄露

`view_image_clip` 将完整图片写入 `/tmp/clipvault-preview-{uuid}.png`，权限 644，永不删除。如果剪贴板包含密码截图或敏感信息，它们永久残留在 `/tmp`。

### C4: 截图临时文件

`screencapture` 写入可预测路径 `/tmp/clipvault-snip-{timestamp}.png`，存在符号链接攻击风险。

### C5: 明文数据库

所有剪贴板内容（可能包含密码、API Key、令牌）以明文存储在 SQLite。任何本地进程可读。

---

## HIGH 问题清单

| # | 问题 | 文件 | 行号 |
|---|------|------|------|
| H1 | `std::sync::Mutex` 与 `tokio::sync::Mutex` 混用 — `blocking_lock` 死锁风险 | state.rs:28, lib.rs:105 |
| H2 | `expect()`/`unwrap()` 在生产路径 — monitor 启动失败直接 panic | lib.rs:109 |
| H3 | `let _ =` 静默忽略错误 — 剪贴板写入失败后仍执行粘贴 | clipboard.rs:43-44 |
| H4 | `reveal_path` 路径遍历 — `open -R <未验证路径>` | clipboard.rs:121-133 |
| H5 | 临时文件累积 — `view_image_clip` 无清理 | clipboard.rs:91-99 |
| H6 | 阻塞 I/O 在 async 上下文 — `std::fs::remove_file` | screenshot.rs:47 |
| H7 | monitor 的 `read_clipboard` 阻塞 tokio worker | monitor_service.rs:101 |
| H8 | Blob URL 内存泄漏 — 从不 `revokeObjectURL` | ClipVaultView.tsx:33-44 |
| H9 | 500 行单体组件 — 无拆分 | ClipVaultView.tsx |
| H10 | ErrorBoundary 失效 — `useState` 无法捕获渲染错误 | App.tsx:14-34 |
| H11 | DRY 违规 — `loadClips` 与 `loadClipsSilent` 近乎相同 | useClips.ts:18-49 |
| H12 | 未使用的 zustand 依赖 | package.json |
| H13 | `shell:allow-open` 权限过于宽泛 | capabilities/default.json:16 |
| H14 | CSP 过于宽松 — `data:/blob:` 在 `default-src` | tauri.conf.json:28 |
| H15 | `create_clip` 无内容大小限制 | clipboard.rs:7-17 |

---

## MEDIUM 问题清单

| # | 问题 | 文件 |
|---|------|------|
| M1 | Clippy 错误未修复（`!is_ok()` → `is_err()`） | clipboard.rs:87 |
| M2 | `cargo fmt` 格式不一致 | 多文件 |
| M3 | `compute_hash` 函数重复定义 | clip_service.rs / monitor_service.rs |
| M4 | `ContentType::from_str` 应实现 `FromStr` trait | models/mod.rs:23 |
| M5 | base64 序列化可用 `serde_with` 替代 | models/mod.rs:86 |
| M6 | 缺少 `#[must_use]` 标注 | 多文件 |
| M7 | `list_clips` 的 limit/offset 无上限校验 | commands/mod.rs:17 |
| M8 | `update_config` 验证不充分 | commands/mod.rs:133 |
| M9 | 错误消息泄露内部信息 | 多文件 |
| M10 | TODO: tray pause monitor 未接线 | lib.rs:79 |
| M11 | `focusIdx` 不随 clips 变化重置 | ClipVaultView.tsx:139 |
| M12 | `formatTime` 不纯（`Date.now()` 在渲染路径） | ClipVaultView.tsx:80 |
| M13 | FTS sanitize 意图与行为不一致 | clip_repo.rs:116 |
| M14 | DB Mutex 无超时 — 配合无限制 create_clip 可 DoS | state.rs:28 |
| M15 | `exclude_sources` 未实现 | monitor_service.rs |
| M16 | 退出确认弹窗无焦点陷阱 | ClipVaultView.tsx:489 |

---

## 正面发现

1. SQL 注入防护到位 — 所有查询使用参数化 `params![]`
2. 无 `unsafe` 代码
3. `AppError` 的 `Serialize` 只返回字符串，不暴露 Rust 类型
4. `ContentType` 通过 serde 枚举验证，不可能传入非法值
5. `freezePrototype: true` 防止原型污染
6. 前端无 `dangerouslySetInnerHTML`
7. `script-src 'self'` 正确限制脚本来源
8. UUID 验证在路径拼接前执行
9. 无硬编码密钥
10. 架构分层合理（commands → services → repositories）

---

## 修复优先级路线图

### Phase 1: 安全加固（1 天）

| 任务 | 级别 | 预估 |
|------|------|------|
| 临时文件：使用 0o600 权限 + 延迟清理 | C3,C4 | 2h |
| 粘贴竞态：添加 paste-in-progress 标志 | C2 | 2h |
| CSP 收紧：`data:/blob:` 限制到 `img-src` | H14 | 30m |
| 移除 `shell:allow-open` 权限 | H13 | 15m |
| `create_clip` 添加 10MB 大小限制 | H15 | 30m |
| `reveal_path` 路径验证 | H4 | 1h |

### Phase 2: 稳定性（1 天）

| 任务 | 级别 | 预估 |
|------|------|------|
| monitor `read_clipboard` 用 `spawn_blocking` 包装 | H7 | 1h |
| `screenshot.rs` 阻塞 I/O 改 `tokio::fs` | H6 | 30m |
| 剪贴板写入失败应返回错误 | H3 | 1h |
| monitor 启动失败改为 log + 降级 | H2 | 30m |
| 修复 clippy 错误 | M1 | 30m |
| 运行 `cargo fmt` | M2 | 15m |
| 提取重复的 `compute_hash` 和 PNG 编码 | M3,L4 | 1h |

### Phase 3: 测试（1 天）

| 任务 | 级别 | 预估 |
|------|------|------|
| Rust 单元测试：clip_service, clip_repo, models | C1 | 3h |
| Rust 集成测试：Tauri commands | C1 | 2h |
| 前端测试：useClips hook, dataUriToBlob, formatTime | C1 | 2h |
| CI 添加 `cargo test` 和 `npm test` | CI | 30m |

### Phase 4: 前端重构（0.5 天）

| 任务 | 级别 | 预估 |
|------|------|------|
| 拆分 ClipVaultView 为 5-6 个组件 | H9 | 2h |
| 修复 ErrorBoundary（用 class component） | H10 | 30m |
| 合并 loadClips/loadClipsSilent | H11 | 30m |
| 修复 Blob URL 内存泄漏 | H8 | 1h |
| focusIdx 重置 + formatTime 纯化 | M11,M12 | 30m |
| 移除未使用的 zustand | H12 | 15m |

### Phase 5: 加密存储（后续迭代）

| 任务 | 级别 | 预估 |
|------|------|------|
| 集成 SQLCipher（`bundled-sqlcipher` feature） | C5 | 1-2 天 |
| 密钥从 macOS Keychain 派生 | C5 | 1 天 |

---

## Skill/Agent 缺口分析

### 现有覆盖情况

| Skill | 状态 | 覆盖范围 |
|-------|------|----------|
| rust-core | 存在 | 错误处理、所有权、并发、测试（通用） |
| rust-arch | 存在 | Tauri 分层架构、IPC 模式 |
| rust-security-skill | 存在 | 安全审计、CSP、Capabilities |
| rust-frontend | 存在 | React+TS+Vite、IPC 封装 |
| rust-qa-skill | 存在 | 验收清单、IPC 契约验证 |
| tdd-workflow | 存在 | 通用 TDD 方法论 |

### 缺失的 Skill

| Skill | 必要性 | 覆盖的审查发现 |
|-------|--------|----------------|
| **rust-tauri-testing** | 高 | C1 — Tauri v2 专属测试模式：`#[tokio::test]` + `state::new_test()`、rusqlite 内存库、前端 vitest + @testing-library |
| **rust-async-patterns** | 中 | H1,H6,H7 — async/await 反模式检测：`spawn_blocking` 使用时机、Mutex 选型（std vs tokio）、避免 `blocking_lock` |

### 需要更新的 Skill

| Skill | 更新内容 |
|-------|----------|
| rust-core | 添加临时文件安全模式（权限、清理、随机名） |
| rust-security-skill | 添加剪贴板管理器专项检查清单 |
| rust-frontend | 添加组件拆分规范（<400 行、单一职责） |

---

## 结论

ClipVault V1 功能基本完备，架构分层清晰，SQL 注入防护到位。但在三个维度存在系统性不足：

1. **安全**：临时文件泄露敏感数据 + 数据库未加密 → Phase 1 优先
2. **稳定性**：竞态条件 + 阻塞 async + 静默错误 → Phase 2
3. **测试**：零覆盖，无法验证任何业务逻辑 → Phase 3

建议按 Phase 1-4 顺序执行，Phase 5（加密存储）作为下个迭代目标。同时补齐 `rust-tauri-testing` 和 `rust-async-patterns` 两个 Skill，防止同类问题再现。
