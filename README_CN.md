# ClipVault

> 轻量级、隐私优先的 macOS 剪贴板历史管理器。

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Platform: macOS](https://img.shields.io/badge/platform-macOS-lightgrey.svg)]()

[English](README.md)

## 特性

- **自动监控剪贴板** — 捕获文本、图片和文件路径
- **全文搜索** — 基于 SQLite FTS5
- **图片缩略图** — 支持预览和在 Finder 中查看
- **收藏功能** — 收藏常用剪贴板条目
- **全局快捷键** — `Cmd+Shift+V` 呼出/隐藏面板
- **系统托盘** — 从菜单栏显示、暂停或退出
- **截图捕获** — 使用 macOS `screencapture`
- **模拟粘贴** — 直接写入当前活跃应用
- **内容去重** — SHA-256 哈希防止重复
- **自动清理** — 按时间或数量自动清除
- **深色 / 浅色主题** — 跟随系统偏好
- **自定义标题栏** — 原生 macOS 交通灯按钮
- **零遥测、零网络** — 完全离线，数据只留在本机

## 技术栈

| 层级   | 技术                                                |
|--------|-----------------------------------------------------|
| 前端   | React 19, TypeScript 5.5, Vite 6, Tailwind CSS 4   |
| 状态   | Zustand 5                                           |
| 后端   | Tauri 2, Rust (edition 2021, MSRV 1.80)             |
| 数据库 | SQLite via rusqlite (bundled + FTS5)                |
| 剪贴板 | arboard (跨平台)                                    |
| 输入   | enigo (模拟粘贴)                                    |

## 前置要求

- macOS 13 (Ventura) 或更高版本
- [Rust](https://rustup.rs/) 1.80+
- [Node.js](https://nodejs.org/) 20+ 和 npm
- Xcode Command Line Tools (`xcode-select --install`)

## 快速开始

### 克隆

```bash
git clone https://github.com/zolo1978/clipvault.git
cd clipvault
```

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 生产构建

```bash
npm run tauri build
```

构建产物（DMG 和 `.app`）在 `src-tauri/target/release/bundle/` 目录。

## 项目结构

```
clipvault/
├── src/                    # React 前端
│   ├── api/                # Tauri IPC 封装
│   ├── hooks/              # React hooks (useClips)
│   ├── views/              # UI 组件
│   ├── lib/                # 工具函数 (theme, safe-invoke)
│   ├── App.tsx             # 根组件 + 错误边界
│   └── main.tsx            # 入口
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── commands/       # Tauri IPC 命令处理
│   │   ├── services/       # 业务逻辑 (clip_service, monitor_service)
│   │   ├── repositories/   # 数据访问层 (rusqlite 查询)
│   │   ├── models/         # 数据模型 + 序列化
│   │   ├── state.rs        # AppState, AppConfig
│   │   ├── error.rs        # 统一错误类型 (thiserror)
│   │   ├── lib.rs          # 插件配置、托盘、快捷键
│   │   └── main.rs         # 二进制入口
│   ├── migrations/         # SQLite 迁移
│   ├── capabilities/       # Tauri 权限清单
│   ├── icons/              # 应用图标
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                   # 设计文档
├── index.html
├── package.json
└── vite.config.ts
```

## 架构

```
React 前端  ←→  Tauri IPC (invoke)  ←→  Rust 命令层
                                                  ↓
                                            服务层 (业务逻辑)
                                                  ↓
                                           仓储层 (SQL 查询)
                                                  ↓
                                             SQLite (FTS5)
```

监控服务以可配置的间隔异步轮询剪贴板。每条捕获的内容通过 SHA-256 去重，存入 SQLite，并通过 Tauri 事件推送到前端。

## 快捷键

| 快捷键          | 操作                  |
|----------------|-----------------------|
| `Cmd+Shift+V`  | 呼出/隐藏面板          |
| `Enter`        | 粘贴选中的条目         |
| `上/下箭头`     | 导航列表              |
| `Escape`       | 清空搜索              |
| 双击            | 按类型执行（粘贴/查看/定位）|

## 配置

ClipVault 通过 `tauri-plugin-store` 存储配置：

| 键                    | 默认值           | 说明                         |
|-----------------------|-----------------|------------------------------|
| `max_clips`           | 10000           | 最大保留条数                  |
| `keep_days`           | 0               | 自动清理 N 天前的条目 (0 = 禁用) |
| `monitor_interval_ms` | 250             | 剪贴板轮询间隔 (最小: 50ms)    |
| `exclude_sources`     | []              | 排除的来源 (计划中)            |
| `shortcut`            | `Cmd+Shift+V`   | 全局快捷键                    |

## 参与贡献

参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解开发指南。

## 许可证

本项目基于 [GNU General Public License v3.0](LICENSE) 开源。

## 致谢

基于 [Tauri](https://tauri.app/)、[React](https://react.dev/)、[Rust](https://www.rust-lang.org/) 和 [Tailwind CSS](https://tailwindcss.com/) 构建。
