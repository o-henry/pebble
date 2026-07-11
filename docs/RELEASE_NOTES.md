# Release Notes

## 0.1.0-pre-alpha

This is a contributor-facing pre-alpha demo build.

### Added

- Tauri 2 desktop scaffold with React, TypeScript, and Rust.
- Hard performance contract for FPS and active tiles; selected regions may use
  any non-empty area inside the active display.
- One-drag region selection that opens a floating Pebble automatically.
- Guarded macOS selected-region capture with a fake backend for tests.
- Capture lifecycle and scheduler with privacy blank behavior.
- Local visual diff engine with cooldown.
- Always-on-top low-FPS live tile with pause, resume, privacy blank, native
  close, and direct menu-bar reopening.
- Config-only persistence for safe region settings.
- Optional local OCR boundary, disabled by default.
- API-key-free OpenAI account connection through a bundled Codex app-server.
- Optional Claude Pro/Max connection through an installed official Claude CLI.
- Explicit one-shot selected-region questions using compact low-effort models,
  with provider, model, and generation-time metadata.
- Local-only material-change alerts with a five-minute cooldown, orange menu-bar
  state, and native notifications.
- Desktop-wide product positioning that covers browsers and native apps rather
  than browser content alone.
- Manual smoke checklist and issue templates.

### Security And Privacy

- No telemetry.
- No cloud sync.
- No frame history.
- No OCR history persistence.
- No AI website automation.
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
