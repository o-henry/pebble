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

The macOS adapter uses Apple Vision locally after a stable Watch change
candidate. Cross Check also performs one disclosed baseline read per enrolled
region. A deterministic intent compiler evaluates common text, single-number
threshold, progress, and state rules locally. Recognized text is ephemeral,
bounded, and treated as untrusted evidence; it is never persisted or used as an
instruction source. Unsupported or ambiguous rules may use the explicitly
enabled AI fallback.

### WatchTargetRegistry

Owns at most three independent runtime-only Watch targets. Each target freezes
its source-window-bound crop, display scale, intent plan, interval, provider,
model, and AI-fallback choice. Reselection changes only the current UI region;
it cannot retarget an existing Watch. Per-target authorization is atomically
revoked on stop, privacy blank, Pebble removal, or app shutdown, so late AI
results cannot notify. Coordinates, window IDs, pixels, and OCR text are never
serialized through Watch status.

For Cross Check targets, the registry stores only a positive, negative, or
unknown state derived from ephemeral local OCR. It forms a conflict candidate
only among explicitly enrolled Cross Check targets and emits after the same
opposing state set survives two additional five-second checks. Raw OCR text and
capture data never enter the registry.

For Follow Through targets, the registry owns one memory-only pending relation:
the trigger target ID, result target IDs still waiting, deadline tick, and safe
region labels. Stable visual changes are the only inputs. A result change clears
that result, all responses clear the relation silently, and expiry emits one
local missed-result signal. Capture failure or target mutation clears the
relation fail-closed. This engine has no OCR, AI, network, or input-control path.

Loop Detector owns a per-target `VisualLoopDetector`. It samples a fixed grid
from baseline and stable-change frames, quantizes each RGB channel and local
contrast to two bits, and retains at most twelve 64-byte fingerprints in a bounded deque. Tail
matching recognizes distinct periods of two through four after three cycles;
canonical cycle rotation prevents duplicate alerts. The detector has no serde
implementation or outbound adapter, and capture failure resets its state.

### AiRuntime

Optional explicit-request service. Manual questions and locally gated Watch
fallback call it through separate authorization paths; it never controls the
capture loop.

Responsibilities:

- Start the fixed bundled Codex app-server or validated installed Claude CLI
  from Rust only when using account access.
- Store an optional Anthropic API key only in macOS Keychain and call only fixed
  Anthropic HTTPS endpoints from Rust.
- Keep provider environments and runtime directories isolated.
- Expose the active account, subscription, or API billing path in the UI.
- Complete official provider login without browser cookie access.
- Capture the backend-selected region once per visible **Send** action.
- Encode the crop to an in-memory PNG data URL.
- Let the user select an available image-capable provider model, then validate
  that choice again in Rust before each medium-effort request.
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
