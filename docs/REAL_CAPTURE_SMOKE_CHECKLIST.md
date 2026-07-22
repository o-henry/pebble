# Real Capture Smoke Checklist

Use this checklist on macOS before enabling real capture in a public build.

## Permission Denied

- Remove Pebble from System Settings > Privacy & Security > Screen
  Recording, or disable its access.
- Start the app and request one real capture for a small selected region.
- Confirm the app reports a recoverable permission error.
- Confirm the app does not crash, retry in a tight loop, or show stale pixels.

## Permission Allowed

- Enable Screen Recording permission for Pebble and restart if macOS asks.
- Select a small region such as 300x180.
- Request one real capture.
- Confirm the returned frame dimensions match the selected region.
- Confirm no screenshot, preview, or temporary capture file appears in the repo,
  app support directory, Downloads, Desktop, or system temp directory.

## Signed Update Continuity

- Start from the previous Developer ID-signed and notarized DMG.
- Confirm its bundle identifier is `com.ohenry.screenpebble`, its Team ID matches
  the candidate, and neither designated requirement contains `cdhash`.
- Approve Screen Recording once and confirm a selected region captures.
- Quit Pebble normally. Do not run `tccutil`, remove the Privacy & Security
  entry, or change its toggle.
- Install the candidate with `npm run install:macos -- /path/to/Pebble.app`.
- Relaunch Pebble and confirm the selected-region flow works without another
  Screen Recording prompt.
- Repeat on clean Intel and Apple Silicon Macs using their matching DMGs.
- Do not promote the prerelease when either architecture asks for permission a
  second time.

## Lifecycle

- Start a live tile and then pause it.
- Confirm capture stops while paused.
- Start a live tile and trigger privacy blank.
- Confirm capture stops while blanked and resumes only after restore.
- Close the tile and quit the app.
- Confirm no capture task remains active.
