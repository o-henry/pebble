# Pebble Interface Contract

## Purpose

Pebble is an operational desktop tool. Its interface exists to complete
one task: select a small screen region and keep that region visible in a compact
always-on-top window.

## Product Flow

1. Pebble launches as a native macOS menu bar utility without a management window.
2. `Select Region...` requests the macOS Screen Recording consent prompt.
3. The selector explains one gesture: drag over the region and release.
4. Releasing a valid selection starts observation and opens one Pebble window
   automatically.
5. The Pebble toolbar owns live, pause, reselect, privacy, and AI.
6. The AI composer expands inside that same window only when requested.

There is no test-tile control, performance inspector, or contributor status in
the user workflow.

## Information Hierarchy

1. The single current task or active region state.
2. Direct controls for the floating window.
3. Compact local-monitoring and privacy state.

Implementation details, hard limits, phase labels, and architecture guidance
belong in documentation rather than the application surface.

## Layout

- Menu bar: left click opens the window; right click offers selection and quit.
- Empty Pebble: one primary action and no visible product title or setup dashboard.
- Active Pebble: compact resizable always-on-top window with the frame as the
  dominant area and a single stable toolbar.
- AI composer: an inline extension of the Pebble, hidden by default.
- Selector: full-display overlay with a fixed instruction HUD and visible drag
  bounds.
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

- The titlebar drag surface invokes native window dragging.
- `Select Region` requests consent before opening the selector.
- A valid pointer release starts the Pebble without an extra confirmation step.
- Escape and the visible close control cancel selection without changing the
  active region.
- The eye control blanks the preview and stops its capture request.
- The AI control expands or collapses the inline provider and question composer.
- Native close stops capture but keeps the current region available to
  reopen from the menu bar.
- Live, pause, reselect, privacy, and AI controls keep stable
  dimensions in the floating window.

## Data Honesty

- Browser development mode is labelled `Preview mode` and never claims that
  desktop capture is running.
- Empty frames are labelled as starting, paused, blanked, or needing attention.
- Only backend-validated physical regions are accepted from cross-window
  events.
- Captured frames remain memory-only and are never represented as stored
  history.
- The interface never suggests cloud sync or active AI.
