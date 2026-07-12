# Implementation Plan

This plan is written for implementation agents. Do one phase at a time. Each
phase should be a small pullable commit with focused tests and review.

## Required Read Order

1. `AGENTS.md`
2. `README.md`
3. `docs/PRODUCT_SPEC.md`
4. `docs/SECURITY_AND_PRIVACY.md`
5. `docs/GIT_AND_SECURITY_POLICY.md`
6. `docs/ENGINEERING_CHARTER.md`
7. `docs/ARCHITECTURE.md`
8. `docs/AI_HANDOFF_DESIGN.md`
9. `docs/DEVELOPMENT_WORKFLOW.md`
10. This file

If a task conflicts with these docs, stop and report the conflict.

## Global Rules

- Keep every phase atomic.
- Do not skip tests for implemented behavior.
- Use fake capture before real capture.
- Do not add telemetry, cloud sync, OCR, AI handoff, or network features unless
  the current phase explicitly calls for them.
- Do not persist captured frames or OCR history.
- Do not broaden Tauri permissions casually.
- Run a review/refactor agent before commit.
- Commit and push after every completed phase.

## Phase 0 - Scaffold

Goal: create the minimal app skeleton without capture.

Build:

- Tauri 2 + React + TypeScript + Vite scaffold.
- Rust backend compiles.
- `src/lib/invoke.ts` exists as the only frontend invoke wrapper.
- Basic app shell renders pre-alpha status and docs links.
- Package scripts for lint, typecheck, test, and dev.

Do not build:

- Screen capture.
- Region selector.
- OCR.
- AI connector.
- Network features.

Checks:

- Frontend lint.
- TypeScript typecheck.
- Frontend tests, even if only smoke tests.
- Rust tests.
- Rust format check.

Commit message:

```text
chore: scaffold desktop app
```

## Phase 1 - Performance Limits

Goal: establish hard product limits before capture exists.

Build:

- Rust `PerformanceLimits` model.
- Validation for FPS, active tile count, and non-empty region size.
- Typed error codes for limit failures.
- Frontend type mirrors generated or manually synchronized in one place.

Tests:

- Default FPS is 1.
- Max FPS is 5.
- Max active tiles is 3.
- Any non-empty region inside the selected display is accepted.
- Invalid values produce typed recoverable errors.

Commit message:

```text
feat: add performance limit validation
```

## Phase 2 - Window Shell

Goal: menu bar control and one test Pebble window without capture.

Build:

- Native menu bar control.
- Test Pebble window.
- Always-on-top tile settings.
- Tile close cleanup event.
- Basic tile states with fake placeholder content.

Do not build capture loops.

Tests:

- UI renders empty state.
- Tile state reducer or model handles live, paused, hidden, blanked, error, and
  closed.
- Close event updates state.

Commit message:

```text
feat: add window shell
```

## Phase 3 - Region Selector Model

Goal: make selection math testable before OS capture.

Build:

- Region selection model.
- Logical-to-physical coordinate mapper.
- Minimum region validation.
- Display-bound and non-empty selection messaging.

Tests:

- Normal selection.
- Reversed drag direction.
- Non-empty region validation.
- Full-display selection acceptance.
- Scale factor conversion.
- Multi-monitor offset cases.

Commit message:

```text
feat: add region selection model
```

## Phase 4 - Region Selector UI

Goal: let the user select a region, still without real capture.

Build:

- Transparent selector overlay shell.
- Drag interaction.
- Dimension display.
- Escape cancel.
- Too-large warning.
- Return validated physical region.

Tests:

- Selector reducer or interaction model.
- Escape cancel behavior.
- Warning appears for large regions.

Commit message:

```text
feat: add region selector overlay
```

## Phase 5 - Fake Capture Backend

Goal: make capture flows testable without OS permissions.

Build:

- `CaptureBackend` trait.
- `FakeCaptureBackend`.
- `capture_region_once` using fake deterministic frames.
- Cropped frame payload type.

Tests:

- Fake backend returns deterministic frames.
- Out-of-bounds region returns typed error.
- No file writes occur.
- Payload represents cropped content only.

Commit message:

```text
feat: add fake capture backend
```

## Phase 6 - Capture Lifecycle And Scheduler

Goal: prove capture stops in inactive states before real capture.

Build:

- `CaptureLifecycle`.
- `CaptureScheduler`.
- Task registry.
- Pause, hide, blank, close, delete transitions.
- Buffer cleanup.

Tests:

- Capture starts only when live and visible.
- Paused tile does not capture.
- Hidden tile does not capture.
- Privacy blank stops all capture.
- Close and delete remove tasks.
- Repeated pause/resume does not leak tasks.

Commit message:

```text
feat: add capture lifecycle scheduler
```

## Phase 7 - Diff Engine

Goal: local visual change detection with no stored frames.

Build:

- Downsample-to-grayscale utility.
- Mean absolute difference score.
- Threshold and cooldown policy.
- Changed event from fake frames.

Tests:

- Identical frames score zero.
- Small changes stay below default threshold.
- Large changes cross threshold.
- Cooldown suppresses repeated alerts.
- Only one previous small frame is retained per tile.

Commit message:

```text
feat: add local diff engine
```

## Phase 8 - Privacy Blank And Hotkey Shell

Goal: make trust controls visible before real capture.

Build:

- Privacy banner.
- Blank all tiles action.
- Restore previous active states.
- Hotkey abstraction shell, but only request permissions needed for the current
  platform and phase.

Tests:

- Blank changes lifecycle state.
- Blank stops scheduler through lifecycle checks.
- Restore restarts only previously active tiles.
- UI shows blank state.

Commit message:

```text
feat: add privacy blank controls
```

## Phase 9 - Real Capture Backend

Goal: add one real platform capture adapter behind `CaptureBackend`.

Premortem required before implementation:

- Permission failure behavior.
- Full-frame temporary memory behavior.
- Crop-before-encode strategy.
- Cleanup on close and blank.
- Manual smoke plan.

Build:

- Platform adapter behind `CaptureBackend`.
- Recoverable permission-denied errors.
- Cropped frame only across the UI boundary.
- No disk writes.

Tests:

- Unit tests with fake backend remain primary.
- Adapter error mapping tests where possible.
- Tests or review evidence prove real capture emits cropped-region payloads only.
- Tests or review evidence prove no frame, screenshot, preview, or temporary
  capture file is written to disk.
- Manual smoke checklist for real permission behavior.

Commit message:

```text
feat: add real capture backend
```

## Phase 10 - Live Tile

Goal: connect selected regions to visible low-FPS tile updates.

Build:

- Tile subscribes to frame events.
- Tile keeps only latest frame for rendering.
- Pause/resume controls.
- Close cleanup.
- FPS UI cannot exceed hard max.

Tests:

- Tile renders latest frame only.
- Pause stops scheduler.
- Close stops scheduler.
- FPS clamp works in UI and backend.

Commit message:

```text
feat: render live pebble tile
```

## Phase 11 - Config Store

Goal: persist only safe configuration.

Build:

- `PebbleStore`.
- Config serialization.
- Config migration placeholder.
- Restore named regions without restoring any frame data.

Tests:

- Stores config fields only.
- Does not serialize frame data, previews, OCR history, or AI prompts.
- Handles corrupted store with recoverable error.

Commit message:

```text
feat: persist pebble configuration
```

## Phase 12 - Optional Local OCR

Goal: extract text locally after change detection or explicit user request.

Build only after the core tile app is reliable.

Build:

- `OcrEngine` trait.
- Local OCR adapter.
- Text result is ephemeral unless user explicitly copies/exports.
- OCR runs after diff or explicit user action, not continuously by default.

Tests:

- OCR disabled by default.
- OCR adapter can be faked.
- OCR results are not persisted.
- Dedupe suppresses unchanged text.

Commit message:

```text
feat: add optional local ocr
```

## Phase 13 - Explicit AI Region Questions

Goal: let a user explicitly ask OpenAI or Claude about one selected crop without
an API key or automatic AI monitoring.

Build:

- Codex app-server sidecar with isolated OpenAI account login.
- Optional installed official Claude CLI with Pro/Max account login.
- One backend-authorized crop per visible **Send** action.
- Memory-only PNG data URL payload.
- Balanced provider-specific image model selection with medium reasoning effort.
- Ephemeral read-only thread and bounded answer.

Do not build:

- AI website automation.
- Browser cookie scraping.
- Continuous image streaming.
- Whole-screen AI access.
- MCP integration.
- Automatic change-triggered AI calls.

Tests:

- No AI request occurs without **Send**.
- Unauthorized, blanked, stale, or reconfigured regions cannot upload data.
- The image payload is a selected crop encoded without a temp file.
- Expensive non-mini model fallback is rejected.
- Webviews have no shell or opener permission.

Commit message:

```text
feat: add explicit region questions
```

## Phase 14 - Release Readiness

Goal: prepare the first public demo.

Build:

- README aligned with shipped behavior.
- Demo GIF.
- Installation path.
- Manual smoke checklist.
- Issue templates.
- Release notes.

Manual checks:

- Idle CPU with no tile.
- One 600x300 tile at 1 FPS for 60 seconds.
- Three 600x300 tiles at 1 FPS for 60 seconds.
- Pause/resume 20 times.
- Privacy blank 20 times.
- Create/delete tile 20 times.
- Permission denied flow.
- Quit and verify no capture task remains.

Commit message:

```text
docs: prepare first public release
```
