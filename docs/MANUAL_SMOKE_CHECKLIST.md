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
npm run release:check
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cd ..
npm run tauri:build
```

## Public Release Checks

- The tag is exactly `v` plus the synchronized application version.
- Both `arm64` and `x64` jobs use a `Developer ID Application` identity.
- The workflow contains no ad-hoc signing fallback.
- Both app bundles and DMGs pass Apple notarization and stapling validation.
- Gatekeeper accepts each app bundle.
- The GitHub prerelease is created only after both architecture jobs succeed.
- No certificate, private key, password, Apple account value, or captured screen
  content appears in the repository, workflow log, or release notes.
- Installing an update over the previous official build retains Screen
  Recording authorization.

## Manual App Checks

- Launch shows one Pebble icon in the macOS menu bar and no Dock icon.
- Left-clicking the menu-bar icon opens the Pebble directly.
- Right-clicking the menu-bar icon offers `Select Region...` and `Quit`.
- No persistent management window opens alongside the floating Pebble.
- The empty and active Pebble have no visible product title.
- Dragging the white titlebar area moves the window.
- AI opens and closes inside the Pebble without creating another window.
- Collapsing AI returns the Pebble to its compact height.

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
- No manual AI request occurs before **Send**, and no Watch AI request occurs before opt-in and a material local change.
- A fresh app data directory shows **Connect OpenAI** and does not reuse another
  Codex installation's account.
- OpenAI sign-in opens only an exact `https://chatgpt.com` or
  `https://auth.openai.com` host and returns through the hosted success page.
- OpenAI sign-in persists successfully in the macOS login keychain without a
  `persist_failed` or missing-default-keychain error.
- With no Claude API key saved, Claude uses only the installed official CLI and
  its Pro/Max login. If the CLI is absent, Pebble opens only the fixed official
  installation page.
- Adding a valid Anthropic API key marks the Claude path **API Billing** without
  returning the saved key to the webview.
- Relaunching Pebble preserves only the key's configured status through macOS
  Keychain; no key value appears in app files, logs, config, or update journals.
- Replacing the key overwrites the Keychain item, and **Use Subscription**
  removes it and restores the CLI subscription path.
- An invalid or unauthorized saved key shows a generic recoverable error and
  never silently falls back to subscription billing or another model.
- Claude subscription mode uses the selected Sonnet or Opus alias at medium
  effort, no tools, no MCP, one turn, and no image temp file.
- Claude API mode calls only fixed Anthropic HTTPS endpoints, follows no
  redirects, defines no tools, and rejects tool-use responses.
- One question sends one selected crop; no full-screen or temporary image file
  is created.
- Privacy blank, reselection, or display reconfiguration before upload cancels
  the request.
- OpenAI lists only supported image-capable Sol, Terra, and Luna models reported
  by the connected account; selecting a different model never silently falls back.
- Watch runs Apple Vision OCR only after a stable material-change candidate,
  except for each explicitly enrolled Cross Check baseline, and keeps OCR text
  ephemeral.
- Cross Check ignores one enrolled region, compares only other Cross Check
  regions, waits 10 seconds before conflict notification, and never calls AI.
- Follow Start does nothing until at least one Follow Result is active. A result
  change before the selected deadline stays silent; an unchanged result emits
  one linked-region alert at expiry and rearms only after a new trigger change.
- Stopping a linked target or making it unavailable cancels a pending Follow
  Through check without alerting. The path runs no OCR, AI, or input control.
- Loop Detector stays silent for a static baseline, one-off changes, and fewer
  than three complete cycles. Repeating two, three, or four distinct states
  three times emits one alert, remains deduplicated, and rearms after a break.
- Loop status, Updates JSON, and the Markdown journal contain no fingerprint,
  frame bytes, coordinates, OCR text, or model metadata.
- Two or more meaningful signals within five minutes render as one Change
  Story in oldest-to-newest order. A Waiting or Analysis Skipped entry breaks
  the story, and a ninth event starts a new item.
- Expanding and collapsing Updates does not rewrite or reorder the saved
  Markdown journal.
- Watch respects each 1, 5, 30, and 60 minute interval without a fixed session cap.
- A custom composer intent suppresses notifications and journal entries when
  the typed Watch result is unmatched.
- Core monitoring adds no network requirement.

## Demo GIF Checks

- GIF is under 15 seconds.
- GIF shows visible user control, not hidden monitoring.
- GIF does not reveal private screen content.
- GIF shows local-first language or privacy blank.
- GIF is committed only after review.
