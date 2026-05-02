# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in ClipVault, please report it privately:

1. **Do not** open a public GitHub issue.
2. Use [GitHub's private vulnerability reporting](https://github.com/zolo1978/clipvault/security/advisories/new).
3. Include: description, steps to reproduce, and potential impact.

We will acknowledge your report within 48 hours and aim to provide a fix within 7 days.

## Security Design

- **Zero network**: ClipVault makes no network requests. All data stays on your machine.
- **Local storage**: Clipboard data is stored in SQLite at `~/Library/Application Support/clipvault/clipvault.db`.
- **No telemetry**: No analytics, crash reporting, or usage tracking.
- **Minimal permissions**: Tauri capabilities are restricted to clipboard access, global shortcuts, and local file system (app data directory only).
- **Content hashing**: SHA-256 deduplication hashes are non-reversible.
- **SQL injection prevention**: All queries use parameterized statements.

## Known Considerations

- Clipboard contents are stored unencrypted in SQLite. Users handling sensitive content should enable macOS FileVault.
- The "file path" detection checks if clipboard text matches an existing filesystem path. This does not access file contents.
