# Release Notes

## 0.1.0-pre-alpha

This is a contributor-facing pre-alpha demo build.

### Added

- Tauri 2 desktop scaffold with React, TypeScript, and Rust.
- Hard performance contract for FPS, active tiles, and region size.
- Region selection model and selector shell.
- Fake capture backend and guarded macOS capture adapter.
- Capture lifecycle and scheduler with privacy blank behavior.
- Local visual diff engine with cooldown.
- Low-FPS live tile demo path.
- Config-only persistence for safe region settings.
- Optional local OCR boundary, disabled by default.
- Optional AI handoff policy boundary, disabled by default.
- Manual smoke checklist and issue templates.

### Security And Privacy

- No telemetry.
- No cloud sync.
- No frame history.
- No OCR history persistence.
- No ChatGPT web automation.
- No browser cookie, token, or API-key reuse.
- No AI handoff by default.

### Known Limits

- No signed installer yet.
- No Homebrew formula yet.
- End-user real tile creation flow is still pre-alpha.
- Production local OCR adapter is not wired yet.
- Production AI connector is not wired yet.
