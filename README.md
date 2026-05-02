# ClipVault

> A lightweight, privacy-first clipboard history manager for macOS.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Platform: macOS](https://img.shields.io/badge/platform-macOS-lightgrey.svg)]()

[中文文档](README_CN.md)

## Features

- **Automatic clipboard monitoring** — captures text, images, and file paths
- **Full-text search** — powered by SQLite FTS5
- **Image thumbnails** — with preview and reveal in Finder
- **Favorites** — bookmark frequently used clips
- **Global hotkey** — `Cmd+Shift+V` to toggle the panel
- **System tray** — show, pause, or quit from the menu bar
- **Screenshot capture** — via macOS `screencapture`
- **Paste simulation** — writes directly to the active application
- **Content deduplication** — SHA-256 hashing prevents duplicates
- **Auto-purge** — by age or count
- **Dark / Light theme** — follows system preference
- **Custom titlebar** — native macOS traffic lights
- **Zero telemetry, zero network** — fully offline, all data stays on your machine

## Tech Stack

| Layer     | Technology                                          |
|-----------|-----------------------------------------------------|
| Frontend  | React 19, TypeScript 5.5, Vite 6, Tailwind CSS 4   |
| State     | Zustand 5                                           |
| Backend   | Tauri 2, Rust (edition 2021, MSRV 1.80)             |
| Database  | SQLite via rusqlite (bundled + FTS5)                |
| Clipboard | arboard (cross-platform)                            |
| Input     | enigo (paste simulation)                            |

## Prerequisites

- macOS 13 (Ventura) or later
- [Rust](https://rustup.rs/) 1.80+
- [Node.js](https://nodejs.org/) 20+ and npm
- Xcode Command Line Tools (`xcode-select --install`)

## Installation

### Download (Recommended)

1. Go to the [Latest Release](https://github.com/zolo1978/clipvault/releases/latest)
2. Download `ClipVault_0.1.0_aarch64.dmg`
3. Open the DMG and drag ClipVault to Applications

**macOS Gatekeeper bypass**: Right-click the app → Open → Open again. Or go to `System Settings` → `Privacy & Security` → `Open Anyway`.

### Build from Source

```bash
git clone https://github.com/zolo1978/clipvault.git
cd clipvault
npm install
npx tauri build
# The app bundle will be in src-tauri/target/release/bundle/
```

## Development

```bash
git clone https://github.com/zolo1978/clipvault.git
cd clipvault
npm install
npx tauri dev
```

## Project Structure

```
clipvault/
├── src/                    # React frontend
│   ├── api/                # Tauri IPC wrappers
│   ├── hooks/              # React hooks (useClips)
│   ├── views/              # UI components
│   ├── lib/                # Utilities (theme, safe-invoke)
│   ├── App.tsx             # Root component + error boundary
│   └── main.tsx            # Entry point
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri IPC command handlers
│   │   ├── services/       # Business logic (clip_service, monitor_service)
│   │   ├── repositories/   # Data access layer (rusqlite queries)
│   │   ├── models/         # Data models + serialization
│   │   ├── state.rs        # AppState, AppConfig
│   │   ├── error.rs        # Unified error type (thiserror)
│   │   ├── lib.rs          # Plugin setup, tray, hotkey registration
│   │   └── main.rs         # Binary entry point
│   ├── migrations/         # SQLite schema migrations
│   ├── capabilities/       # Tauri permission manifest
│   ├── icons/              # App icons (png, icns, ico)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                   # Design documents
├── index.html
├── package.json
└── vite.config.ts
```

## Architecture

```
React Frontend  ←→  Tauri IPC (invoke)  ←→  Rust Commands
                                                    ↓
                                              Services (business logic)
                                                    ↓
                                           Repositories (SQL queries)
                                                    ↓
                                              SQLite (FTS5)
```

The monitor service runs an async loop polling the clipboard at a configurable interval. Each captured clip is deduplicated via SHA-256, stored in SQLite, and emitted to the frontend via Tauri events.

## Keyboard Shortcuts

| Shortcut       | Action                     |
|----------------|----------------------------|
| `Cmd+Shift+V`  | Toggle ClipVault panel     |
| `Enter`         | Paste selected clip        |
| `Up/Down`       | Navigate clip list         |
| `Escape`        | Clear search               |
| Double-click    | Action by type (paste/view/reveal) |

## Configuration

ClipVault stores configuration via `tauri-plugin-store`:

| Key                  | Default | Description                          |
|----------------------|---------|--------------------------------------|
| `max_clips`          | 10000   | Maximum clips to retain              |
| `keep_days`          | 0       | Auto-purge clips older than N days (0 = disabled) |
| `monitor_interval_ms`| 250     | Clipboard polling interval (min: 50ms) |
| `exclude_sources`    | []      | Sources to skip (planned)            |
| `shortcut`           | `Cmd+Shift+V` | Global hotkey                  |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

## Acknowledgments

Built with [Tauri](https://tauri.app/), [React](https://react.dev/), [Rust](https://www.rust-lang.org/), and [Tailwind CSS](https://tailwindcss.com/).
