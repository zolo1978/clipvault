# Contributing to ClipVault

Thank you for your interest in contributing to ClipVault!

## Development Setup

### Prerequisites

- macOS 13 (Ventura) or later
- [Rust](https://rustup.rs/) 1.80+
- [Node.js](https://nodejs.org/) 20+ and npm
- Xcode Command Line Tools (`xcode-select --install`)

### Getting Started

```bash
git clone https://github.com/zolo1978/clipvault.git
cd clipvault
npm install
npm run tauri dev
```

## Project Architecture

```
Frontend (React/TypeScript)
  ↓ Tauri IPC (invoke)
Commands (thin adapters)
  ↓
Services (business logic)
  ↓
Repositories (SQL queries)
  ↓
SQLite (rusqlite + FTS5)
```

- **Commands** (`src-tauri/src/commands/`) — validate input, delegate to services
- **Services** (`src-tauri/src/services/`) — pure business logic
- **Repositories** (`src-tauri/src/repositories/`) — SQL queries, data access
- **Frontend** (`src/`) — React views, hooks, API wrappers

## Code Style

### Rust

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` — zero warnings required
- MSRV: 1.80
- Follow standard Rust naming conventions

### TypeScript

- Strict mode enabled in `tsconfig.json`
- No unused locals or parameters
- Run `npx tsc --noEmit` to type-check

## Making Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Ensure all checks pass (fmt, clippy, tsc)
5. Commit with a clear message (conventional commits preferred: `feat:`, `fix:`, `refactor:`)
6. Push and open a Pull Request

## Reporting Issues

- Use [Bug Report](https://github.com/zolo1978/clipvault/issues/new?template=bug_report.yml) for bugs
- Use [Feature Request](https://github.com/zolo1978/clipvault/issues/new?template=feature_request.yml) for suggestions

## Roadmap

Potential areas for contribution:

- Internationalization (i18n) support
- Linux and Windows support
- Tags and categories for clips
- Import/export clipboard history
- Wired tray "pause monitor" action
- Unit and integration tests
