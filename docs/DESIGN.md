# ScreenPebble Interface Contract

## Purpose

ScreenPebble is an operational desktop tool. Its interface exists to complete
one task: select a small screen region and keep that region visible in a compact
always-on-top window.

## Product Flow

1. The empty main window presents one primary action: `Select a region`.
2. macOS owns the Screen Recording consent prompt.
3. The selector explains one gesture: drag over the region and release.
4. Releasing a valid selection starts observation and opens the Pebble window
   automatically.
5. The main window becomes session control for show, reselect, privacy blank,
   and stop.

There is no test-tile control, performance inspector, or contributor status in
the user workflow.

## Information Hierarchy

1. Product name and local-only state.
2. The single current task or active region state.
3. Direct controls for the floating Pebble.
4. Compact privacy guarantees.

Implementation details, hard limits, phase labels, and architecture guidance
belong in documentation rather than the application surface.

## Layout

- Main window: a restrained task workspace, not a dashboard.
- Empty state: purpose, one primary action, and a short set of concrete use
  cases.
- Active state: selected dimensions, monitor, position, window state, and the
  smallest useful command set.
- Selector: full-display overlay with a fixed instruction HUD and visible drag
  bounds.
- Pebble: compact resizable always-on-top window with the frame as the dominant
  area.
- Narrow windows: one deliberate column with no overlapping controls or text.

## Typography

- `DM Mono Nerd Font` is the interface face for English, numeric values,
  status labels, and controls.
- `Apple SD Gothic Neo`, `Pretendard`, and system sans-serif fonts are fallbacks
  for Korean glyphs unavailable in DM Mono Nerd Font.
- Medium weight carries controls and labels, regular carries supporting text,
  and light is reserved for the main product statement.
- Letter spacing remains zero. Numbers use stable tabular widths.

## Visual System

- Canvas: cool neutral gray. Working surfaces: white and near-white.
- Ink: charcoal. Blue marks the primary action, green marks active local
  capture, amber marks privacy blank, and red is reserved for recoverable
  errors or destructive commands.
- Panels use 8px or smaller corners, 1px dividers, and no decorative gradients,
  floating blobs, or nested card stacks.
- Every color state also has a concise text label.

## Interaction Rules

- `Select a region` requests macOS consent before opening the selector.
- A valid pointer release starts the Pebble without an extra confirmation step.
- Escape and the visible close control cancel selection without changing the
  active region.
- `Hide preview` blanks the floating tile and stops its capture request.
- Closing the Pebble stops capture but keeps the current region available to
  reopen.
- `Stop watching` closes the Pebble and clears the in-memory session region.
- Live, pause, refresh, and close controls keep stable dimensions in the
  floating window.

## Data Honesty

- Browser development mode is labelled `Preview mode` and never claims that
  desktop capture is running.
- Empty frames are labelled as starting, paused, blanked, or needing attention.
- Only backend-validated physical regions are accepted from cross-window
  events.
- Captured frames remain memory-only and are never represented as stored
  history.
- The interface never suggests cloud sync or active AI.
