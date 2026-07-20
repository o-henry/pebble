# Release Notes

## 0.2.0-pre-alpha

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
- Ephemeral Apple Vision OCR behind the stable Watch change gate.
- API-key-free OpenAI account connection through a bundled Codex app-server.
- Claude access through an installed official Claude CLI subscription or an
  optional Anthropic API key stored only in macOS Keychain, with the active
  billing path shown in the UI.
- Explicit one-shot selected-region questions using a user-selected,
  account-validated medium-effort model, with provider, model, and
  generation-time metadata.
- Intent-aware material-change alerts with selectable 1, 5, 30, or 60 minute
  maximum AI cadence, orange menu-bar state, and native notifications.
- Local text, single-number threshold, progress, and state Watch rules that can
  run with no AI connection or token usage.
- Local No Progress Watch that alerts once when a previously active region
  remains stable for the selected interval, without OCR, AI, or network use.
- Local Cross Check Watch that compares coarse positive and negative states
  across two or three explicitly enrolled browser or native-app regions, with a
  fixed 10-second confirmation and no AI use.
- Local Follow Through Watch that links a trigger to one or two result regions
  and reports only downstream regions that miss the selected response deadline,
  without OCR, AI, network access, or input control.
- Local Loop Detector that recognizes 2- to 4-step visual cycles after three
  repetitions using a bounded memory-only fingerprint history with no OCR or AI.
- Frontend-only Change Story timelines that group two to eight nearby meaningful
  Watch results while leaving operational messages and the source journal intact.
- Structured Watch signals with safe region, event, engine or model,
  confidence, and duration metadata separated from the human summary.
- Stable-candidate animation suppression and per-region semantic event dedupe.
- Up to three independently source-bound Watch regions with stable labels,
  individual stop actions, and background operation while Pebble is hidden.
- Privacy-safe local Watch recipes containing only names, intents, and
  recommended intervals.
- Desktop-wide product positioning that covers browsers and native apps rather
  than browser content alone.
- Manual smoke checklist and issue templates.
- Fail-closed Intel and Apple Silicon release automation that requires a stable
  Developer ID identity, Apple notarization, stapling, and Gatekeeper checks
  before publishing either DMG.
- An explicit macOS 14 minimum deployment target for packaged builds.

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

- No signed installer is published until the repository owner configures the
  required Apple Developer certificate and notarization secrets.
- No Homebrew formula yet.
- Multi-monitor selection is still limited to the display containing the main
  window.
- Source-window behavior across every macOS Space, full-screen, minimize, and
  mixed-DPI display transition still needs broader real-device coverage.
