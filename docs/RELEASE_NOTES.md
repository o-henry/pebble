# Release Notes

## 0.1.0-pre-alpha

This is a contributor-facing pre-alpha demo build.

### Added

- Tauri 2 desktop scaffold with React, TypeScript, and Rust.
- Hard performance contract for FPS, active tiles, and region size.
- One-drag region selection that opens a floating Pebble automatically.
- Guarded macOS selected-region capture with a fake backend for tests.
- Capture lifecycle and scheduler with privacy blank behavior.
- Local visual diff engine with cooldown.
- Always-on-top low-FPS live tile with pause, resume, privacy blank, and close.
- Config-only persistence for safe region settings.
- Optional local OCR boundary, disabled by default.
- API-key-free ChatGPT account connection through a bundled official Codex
  app-server.
- Explicit one-shot selected-region questions using a compact image model.
- Manual smoke checklist and issue templates.

### Security And Privacy

- No telemetry.
- No cloud sync.
- No frame history.
- No OCR history persistence.
- No ChatGPT web automation.
- No browser cookie, token, or API-key reuse.
- No automatic AI requests; one crop is sent only after **Ask**.
- No inherited API-key environment, browser cookie access, MCP, web search, or
  webview shell permission.

### Known Limits

- No signed installer yet.
- No Homebrew formula yet.
- Multi-monitor selection is still limited to the display containing the main
  window.
- Production local OCR adapter is not wired yet.
