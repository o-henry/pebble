# Product Spec

Pebble is a local-first desktop app for pinning a small user-selected
screen region as a low-FPS always-on-top tile. It helps users stop babysitting
small visual states without turning the app into a screen recorder, hidden
monitor, or AI surveillance tool.

## One-Line Promise

Pin a tiny part of your screen. Let local watchers notice what changed.

## Target Users

- Developers watching build logs, test output, CI panels, local servers, or
  deployment status.
- Operators watching dashboard cells, queues, timers, or long-running jobs.
- Designers and QA users watching a focused area of a browser or desktop app.
- Power users who want ambient monitoring without cloud integrations.

## Primary Job

The user wants to keep track of a small visible region while doing other work.

Success looks like:

1. The user selects a small region.
2. The app creates a small always-on-top tile.
3. The tile updates at low FPS.
4. Local change detection notices meaningful visual changes.
5. The user can pause, blank, hide, or delete the tile at any time.
6. No captured content is stored or uploaded by default.

## MVP Scope

The first implementation must focus on a trustworthy non-AI desktop utility:

- Tauri 2 desktop scaffold.
- React + TypeScript UI.
- Rust backend services.
- Native macOS menu bar control and one floating Pebble window.
- Region model and performance limits.
- Fake capture backend for tests.
- Capture lifecycle service using fake frames.
- Region selector shell.
- Always-on-top tile shell.
- Local diff engine.
- Privacy blank state.
- Config-only store.

Real OS capture should come only after limits, lifecycle, fake backend tests, and
privacy blank behavior are already in place.

## Explicit Non-Goals

Do not implement:

- Screen recording.
- Frame history.
- Timeline or replay.
- Cloud sync.
- Telemetry or analytics.
- AI watching the whole screen.
- Continuous image streaming to AI.
- Browser cookie scraping.
- ChatGPT web automation.
- Stock, brokerage, or trading integration.
- Hidden capture or workplace-policy bypass features.

## User-Facing States

Every tile must be explainable through explicit states:

- `Live`: visible and actively refreshing.
- `Paused`: visible but not capturing.
- `Hidden`: not shown and not capturing.
- `Blanked`: privacy blank active and not capturing.
- `Closed`: tile window closed and capture task removed.
- `Error`: recoverable issue such as permission denied or monitor unavailable.

AI-related state is separate and must default to off:

- `AI Off`: no handoff can occur.
- `AI Text Ready`: local OCR text is available for explicit or configured
  handoff.
- `AI Enabled`: region is allowed to hand off compact text or explicitly allowed
  images according to its settings.

## Core User Flows

### Create Tile

1. User chooses "New Pebble".
2. Region selector opens.
3. User drags a region.
4. Selector accepts any non-empty area inside the selected display.
5. Backend validates display bounds and positive dimensions.
6. App creates config and opens a tile.
7. Capture starts only after tile is visible.

### Pause Or Blank

1. User clicks pause or global privacy blank.
2. Capture scheduler stops affected loops.
3. Tile state updates visibly.
4. Resume restarts only previously active tiles.

### Local Change Alert

1. Capture scheduler receives latest crop.
2. Diff engine compares a small downsampled frame.
3. If threshold and cooldown pass, tile shows a changed badge.
4. No screenshot or previous full frame is stored.

### Optional AI Handoff

1. User enables AI for a specific region.
2. Local watcher detects a meaningful change.
3. Local OCR extracts text if OCR is implemented.
4. Dedupe and cooldown suppress repeated updates.
5. Connector can hand off compact text, not continuous images.

## Trust Requirements

- Capture is visible.
- Capture stops when the tile is not actively useful.
- The user can blank everything quickly.
- Permission-denied is recoverable.
- Sensitive actions have typed errors and clear user copy.
- The app never implies it can safely bypass OS, DRM, workplace, or platform
  restrictions.

## Acceptance For First Public Demo

The first demo is acceptable when:

- The README accurately matches shipped behavior.
- A user can create and close one tile without leaked capture loops.
- Privacy blank visibly stops capture.
- Default refresh is 1 FPS.
- Max refresh is enforced at 5 FPS.
- No captured frames are written to disk.
- No network request happens during core monitoring.
- The GIF can show the product in under 15 seconds.
