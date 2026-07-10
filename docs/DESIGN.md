# ScreenPebble Interface Contract

## Purpose

ScreenPebble is an operational desktop tool, not a landing page. The main
window must make the current observation state, privacy control, and next
direct action clear before any implementation or product background.

## Information Hierarchy

1. The persistent header identifies the app and its local-only session state.
2. The command deck exposes the two primary actions: selecting a region and
   opening a test tile.
3. The live preview is the dominant work surface.
4. Region selection, tile diagnostics, and hard performance limits form a
   secondary inspector rail.
5. Trust constraints are visible last as a compact reference strip.

## Layout

- Main window: a single restrained workspace with a 1.48fr primary pane and a
  0.82fr inspector rail on desktop.
- Narrow windows: one deliberate vertical column; action buttons remain
  visible and never overlap content.
- The region selector is a full-screen dark overlay with a compact fixed HUD.
- The test tile is a distinct compact dark surface so it reads as a pinned
  object rather than a smaller copy of the control window.

## Typography

- `DM Mono Nerd Font` is the interface face for English, numeric values,
  status labels, and icons.
- `Apple SD Gothic Neo`, `Pretendard`, and system sans-serif fonts are fallback
  faces for Korean glyphs not available in DM Mono Nerd Font.
- Medium weight carries controls and labels; regular carries supporting text;
  light is reserved for large display text.
- Letter spacing remains neutral. Numbers have a stable tabular width.

## Visual System

- Canvas: cool green-gray, not pure white.
- Surfaces: near-white with one consistent 1px divider color.
- Ink: charcoal. Teal denotes an active local signal; amber denotes privacy
  blank; red is reserved for recoverable errors.
- Panels use square-to-subtle 8px corners. There are no decorative gradients,
  floating blobs, or nested card stacks.
- A status dot, border change, and concise label carry state together; color is
  never the only signal.

## Interaction Rules

- `New pebble` always opens the explicit region selector.
- `Privacy blank` remains visible at the top of the workspace and changes to a
  restore command while active.
- Tile controls keep a stable arrangement for live, pause, refresh rate, and
  close actions.
- Region selection can be cancelled with the visible close control or Escape.
- Hover, focus-visible, active, and error states are defined for interactive
  controls.

## Data Honesty

- Empty frame state is labelled as empty or waiting; no synthetic preview data
  is presented as captured content.
- Only actual state from the current frontend model is displayed.
- The interface never suggests screen recording, cloud sync, or active AI.
