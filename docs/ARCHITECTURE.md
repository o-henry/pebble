# Architecture

Pebble should be implemented as a small desktop system with explicit
ownership boundaries. UI renders state and collects intent. Rust services own
capture, lifecycle, limits, diffing, and persistence policy.

## Planned Stack

- Desktop shell: Tauri 2.
- Frontend: React + TypeScript + Vite.
- Backend: Rust.
- Tests: Vitest for TypeScript, Rust unit tests for services.
- Real capture backend: deferred until fake backend and lifecycle tests exist.

## Layering

```text
React UI
  -> typed frontend command wrappers
    -> Tauri command boundary
      -> Rust app services
        -> OS adapters
```

Rules:

- React does not own capture loops.
- Rust services do not depend on React.
- Tauri commands use typed request/response contracts.
- Frontend invokes backend only through `src/lib/invoke.ts`.
- Capture, OCR, notifications, and AI handoff live behind adapters.

## Suggested Repository Shape

```text
src/
  app/
    App.tsx
    routes.tsx
  features/
    pebble/
      PebbleList.tsx
      PebbleTile.tsx
      TileControls.tsx
      pebbleTypes.ts
    region-selector/
      RegionSelectorOverlay.tsx
      regionMath.ts
    privacy/
      PrivacyBanner.tsx
    settings/
      SettingsPanel.tsx
  lib/
    invoke.ts
    events.ts
    result.ts
  styles/
    tokens.css
    app.css
src-tauri/
  src/
    commands/
    models/
    services/
    platform/
    tests/
```

Keep this structure flexible during scaffold, but preserve the ownership
boundaries.

## Domain Model

Use physical pixels for coordinates crossing the frontend/backend boundary unless
the type name explicitly says `Logical`.

```text
PebbleConfig
  id
  name
  region: PhysicalRegion
  window: PebbleWindowConfig
  capture: CaptureConfig
  alert: AlertConfig
  ai: AiHandoffConfig
  createdAt
  updatedAt
```

```text
PhysicalRegion
  monitorId
  x
  y
  width
  height
```

```text
CaptureMode
  live
  paused
  hidden
  blanked
  error
  closed
```

`AiHandoffConfig` must default to disabled.

## Rust Service Responsibilities

### PerformanceLimits

Single source of truth:

- Default FPS: 1.
- Max FPS: 5.
- Max active tiles: 3.
- Region size: any non-empty area inside the selected display.

### RegionMapper

Converts logical screen selection into physical pixels. It must be isolated and
unit-tested because mixed-DPI and multi-monitor cases are easy to break.

### CaptureBackend

Trait for region capture.

Responsibilities:

- List monitors.
- Capture a validated region.
- Return cropped image bytes in memory.

Non-responsibilities:

- Scheduling.
- Lifecycle policy.
- Alerts.
- Storage.
- AI handoff.

### CaptureScheduler

The only owner of capture loops.

Responsibilities:

- Start and stop tile tasks.
- Enforce active tile limits.
- Clamp FPS.
- Apply backoff.
- Stop on paused, hidden, blanked, closed, or deleted.
- Emit frame and error events.
- Drop buffers when a loop stops.

### CaptureLifecycle

Tracks tile state and provides the scheduler with a single question:

```text
should this tile currently capture?
```

### DiffEngine

Computes local visual change score from small downsampled frames.

MVP algorithm:

1. Downsample to 64x64 grayscale.
2. Compare previous and current small frames.
3. Score mean absolute difference normalized to 0.0-1.0.
4. Keep only one previous small frame per tile in memory.

### PebbleStore

Persists config only.

Never store:

- Frames.
- Screenshots.
- OCR history.
- Previews.
- Browser URLs.
- Clipboard contents.
- AI prompts derived from screen content.

### OcrEngine

Future optional adapter. It must be local and run only after local change
detection or explicit user action.

### AiRuntime

Optional explicit-request service. It is never part of the capture loop.

Responsibilities:

- Start the fixed bundled Codex app-server or validated installed Claude CLI
  from Rust only.
- Keep provider environments and runtime directories isolated.
- Complete official provider login without browser cookie access.
- Capture the backend-selected region once per visible **Ask** action.
- Encode the crop to an in-memory PNG data URL.
- Select an image-capable balanced provider model at medium reasoning effort.
- Create an ephemeral read-only thread and reject all tool activity.
- Return a bounded text answer without persistence.

## Command Boundary

Initial commands should stay narrow:

```text
get_performance_limits
list_monitors
create_pebble
update_pebble
delete_pebble
start_capture
stop_capture
set_privacy_blank
capture_region_once
get_ai_connection_status
connect_ai_provider
ask_selected_region
```

All commands return typed success or typed recoverable errors. Do not throw raw
strings across the boundary.

## Event Boundary

Initial events:

```text
pebble://frame-updated
pebble://changed
pebble://monitor-insight
pebble://capture-error
pebble://privacy-mode-changed
pebble://performance-backoff
```

Frame events may carry only cropped tile content. Never emit a full-monitor
frame.

## Backoff Policy

```text
recently changed           -> configured FPS, default 1 FPS
unchanged for 30 seconds   -> max 1 FPS
unchanged for 5 minutes    -> 0.2 FPS or paused until interaction
hover or explicit refresh  -> immediate refresh
paused/hidden/blanked      -> stopped
closed/deleted             -> task removed
```

## Failure Modes To Design For

- Screen capture permission denied.
- Monitor disconnected.
- Region out of bounds after display change.
- Mixed-DPI coordinate mismatch.
- Tile closed while capture task is running.
- Privacy blank toggled during capture.
- Backend unavailable.
- Store write failure.
- AI connector disabled or unauthorized.

Each failure should be recoverable where possible and testable with fake
adapters.
