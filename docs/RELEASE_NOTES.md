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
- Claude access through an installed official Claude CLI subscription or an
  optional Anthropic API key stored only in macOS Keychain, with the active
  billing path shown in the UI.
- Explicit one-shot selected-region questions using balanced medium-effort models,
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
- Manual AI sends one crop only after **Send**; semantic Watch sends bounded
  before/after crops only after an opt-in local material-change gate.
- No inherited API-key environment, browser cookie access, MCP, web search, or
  webview shell permission. Optional Claude API credentials remain in macOS
  Keychain and are never returned to the webview or written to app files.

### Known Limits

- No signed installer yet.
- No Homebrew formula yet.
- Multi-monitor selection is still limited to the display containing the main
  window.
- Production local OCR adapter is not wired yet.
