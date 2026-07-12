# Demo

The README uses `docs/assets/pebble-demo.gif`.

The committed GIF was recorded from the actual Pebble macOS development build.
It shows Pebble keeping a user-selected region of a real TradingView chart
visible in its live window. It is not a mockup or synthetic animation.

The source workspace used a private Chrome window with no signed-in account.
The published crop excludes the macOS menu bar, browser controls, other apps,
and the rest of the desktop. It contains no prompts, tokens, cookies, account
names, or private files.

## Recording Rules

When replacing the GIF with a new real recording:

- Record a clean demo workspace only.
- Keep the capture under 15 seconds.
- Show selected-region behavior and privacy controls.
- Do not show private files, browser sessions, account names, tokens, or API
  keys.
- Run the manual smoke checklist before committing the replacement.

## Current Capture

- Platform: macOS
- App: actual Pebble development build
- Source surface: TradingView in a private Chrome window
- Published size: 800 x 700 pixels
- Duration: 10 seconds
- Frame rate: 8 FPS
- Scope: browser chart and Pebble live window only

## Suggested Storyboard

1. Open Pebble.
2. Show selected-region-only framing.
3. Show a small live tile.
4. Toggle privacy blank.
5. Ask one short question about the selected crop and show the concise answer.
