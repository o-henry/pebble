# Manual Smoke Checklist

Run this before tagging a public demo or recording a new demo GIF.

## Environment

- macOS desktop target.
- Fresh checkout.
- No private `.env` files.
- No captured screen content committed.
- Network disabled or monitored when validating local-only behavior.

## Build Checks

```bash
npm install
npm test
npm run typecheck
npm run lint
npm run build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cd ..
npm run tauri:build
```

## Manual App Checks

- Launch shows one Pebble icon in the macOS menu bar and no Dock icon.
- `Select Region...`, `Show Pebble`, and `Quit Pebble` work from the menu bar.
- No persistent management window opens alongside the floating Pebble.
- ChatGPT opens and closes inside the Pebble without creating another window.
- Collapsing ChatGPT returns the Pebble to its compact height.

- Idle CPU with no tile.
- One 600x300 tile at 1 FPS for 60 seconds.
- Three 600x300 tiles at 1 FPS for 60 seconds.
- Pause/resume 20 times.
- Privacy blank 20 times.
- Create/delete tile 20 times.
- Permission denied flow.
- Built-in Retina selection produces a live frame at the expected crop.
- Retina secondary display with a non-zero origin produces the expected crop.
- Hide the app with an active tile and verify capture stops until it is visible.
- Rearrange or disconnect the selected display and verify capture fails closed.
- Trigger privacy blank and close while capture is active; no late frame appears.
- Quit and verify no capture task remains.

## Privacy Checks

- No frame files written to the repo.
- No screenshots, OCR output, prompts, tokens, cookies, or local account data in
  logs, docs, fixtures, or tests.
- Captured frame payloads remain cropped and memory-only.
- Paused, hidden, blanked, closed, and deleted states do not capture.
- No AI request occurs before a visible **Ask** action.
- A fresh app data directory shows **Connect ChatGPT** and does not reuse another
  Codex installation's account.
- ChatGPT sign-in opens only an exact `https://chatgpt.com` or
  `https://auth.openai.com` host and returns through the hosted success page.
- ChatGPT sign-in persists successfully in the macOS login keychain without a
  `persist_failed` or missing-default-keychain error.
- One question sends one selected crop; no full-screen or temporary image file
  is created.
- Privacy blank, reselection, or display reconfiguration before upload cancels
  the request.
- A subscription without a compatible compact image model fails closed instead
  of selecting a larger model.
- OCR remains disabled by default.
- Core monitoring adds no network requirement.

## Demo GIF Checks

- GIF is under 15 seconds.
- GIF shows visible user control, not hidden monitoring.
- GIF does not reveal private screen content.
- GIF shows local-first language or privacy blank.
- GIF is committed only after review.
